use std::sync::Mutex;

use credit_store_demo::{db, drivers};
use diesel::SqliteConnection;
use log::*;
use shi::{cmd, error::ShiError, parent};

struct InternalShellState {
    _conn: SqliteConnection,
}

struct _ExternalShellState {}

static _G_EXT_SHELL_STATE: Mutex<_ExternalShellState> = Mutex::new(_ExternalShellState {});

fn on_shell_update(_frame: usize) -> Option<()> {
    Some(())
}

fn credit_store_events_insert(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    // G_EXT_SHELL_STATE.lock().unwrap().counting = true;

    let _person = match drivers::read_str_or_quit("person") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let _credits: i32 = match drivers::read_input_from_user_until_valid_or_quit("credits (i32)") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let _event_stack_level: i32 =
        match drivers::read_input_from_user_until_valid_or_quit("event_stack_level (i32)") {
            Some(item) => item,
            None => return Ok("".to_owned()),
        };

    todo!()
}

fn main() {
    info!("Starting demo!");

    drivers::logging::init_logging_with_level(log::LevelFilter::Trace);

    let shell_join = drivers::shell::spawn_shell_loop_thread(
        || InternalShellState {
            _conn: db::loader::establish_connection().unwrap(),
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

    shell_join.join().unwrap().unwrap();
}
