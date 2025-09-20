use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc::channel},
};

use credit_store_demo::{
    db::event_accumulator_actions::{CreditStoreEventSourcable, HeadTable},
    *,
};

use log::*;
use strum::VariantArray;

struct InternalShellState {}

struct _ExternalShellState {}

static _G_EXT_SHELL_STATE: Mutex<_ExternalShellState> = Mutex::new(_ExternalShellState {});

fn on_shell_update(_frame: usize) -> Option<()> {
    Some(())
}

fn main() {
    info!("Starting demo!");

    drivers::logging::init_logging_with_level(log::LevelFilter::Trace);

    let shell_join = drivers::shell::spawn_shell_loop_thread(
        || InternalShellState {},
        Vec::new,
        on_shell_update,
    );

    let (_tx_ea_msg, rx_ea_msg) = channel();
    let (tx_ea_work_done, _rx_ea_work_down) = channel();

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
        rx_ea_msg,
        tx_ea_work_done.clone(),
        table_head_access_lock_map,
        event_sourcable_map,
    );

    shell_join.join().unwrap().unwrap();

    ea_join.join().unwrap().unwrap();
}
