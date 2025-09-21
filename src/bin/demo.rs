use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

use credit_store_demo::{
    db::{
        event_accumulator::{EaCommand, EaWorkError},
        event_accumulator_actions::{CreditStoreEventSourcable, HeadTable},
        models::CreditStoreObject,
    },
    *,
};

use diesel::SqliteConnection;
use log::*;
use shi::{cmd, error::ShiError, parent};
use strum::VariantArray;

struct InternalShellState {
    conn: SqliteConnection,
    tx_cmd_to_ea: Sender<EaCommand>,
    rx_ea_work_done: Receiver<Result<String, EaWorkError>>,
}

struct _ExternalShellState {}

static _G_EXT_SHELL_STATE: Mutex<_ExternalShellState> = Mutex::new(_ExternalShellState {});

fn on_shell_update(_frame: usize) -> Option<()> {
    Some(())
}

fn credit_store_events_insert(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    // G_EXT_SHELL_STATE.lock().unwrap().counting = true;

    let person = match drivers::read_str_or_quit("person") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let credits: i32 = match drivers::read_input_from_user_until_valid_or_quit("credits (i32)") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let event_stack_level: i32 =
        match drivers::read_input_from_user_until_valid_or_quit("event_stack_level (i32)") {
            Some(item) => item,
            None => return Ok("".to_owned()),
        };

    let object = CreditStoreObject { person, credits };

    let res = db::event_accumulator_actions::credit_store_event_blocking_insert(
        &mut mut_state.conn,
        &mut_state.tx_cmd_to_ea,
        &mut_state.rx_ea_work_done,
        &object,
        event_stack_level,
    );

    match res {
        Ok(event) => Ok(format!("Created event {event:?}")),
        Err(e) => Ok(format!("Failed to create event: {e:?}")),
    }
}

fn main() {
    info!("Starting demo!");

    drivers::logging::init_logging_with_level(log::LevelFilter::Trace);

    let (tx_cmd_to_ea, rx_cmd_to_ea) = channel();
    let (tx_ea_work_done, rx_ea_work_done) = channel();

    let shell_join = drivers::shell::spawn_shell_loop_thread(
        || InternalShellState {
            conn: db::loader::establish_connection().unwrap(),
            tx_cmd_to_ea,
            rx_ea_work_done,
        },
        || {
            vec![parent!(
                "db",
                parent!(
                    "credit_store",
                    cmd!(
                        "insert",
                        "insert a new credit store event",
                        credit_store_events_insert
                    )
                )
            )]
        },
        on_shell_update,
    );

    let table_head_access_lock_map = HeadTable::VARIANTS
        .iter()
        .map(|variant| (variant.to_string(), Arc::new(Mutex::new(()))))
        .collect::<HashMap<_, _>>();

    let event_sourcable_map = HeadTable::VARIANTS
        .iter()
        .map(|variant| {
            (
                variant.to_string(),
                match variant {
                    HeadTable::CreditStore => CreditStoreEventSourcable {},
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let ea_join = db::event_accumulator::spawn_ea_bg_worker_thread(
        rx_cmd_to_ea,
        tx_ea_work_done.clone(),
        table_head_access_lock_map,
        event_sourcable_map,
    );

    shell_join.join().unwrap().unwrap();

    ea_join.join().unwrap().unwrap();
}
