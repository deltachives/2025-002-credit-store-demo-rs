use std::sync::Mutex;

use credit_store_demo::*;
use shi::{cmd, error::ShiError, parent};

use log::*;

struct InternalShellState {}

struct ExternalShellState {
    counting: bool,
    counter: usize,
}

static G_EXT_SHELL_STATE: Mutex<ExternalShellState> = Mutex::new(ExternalShellState {
    counting: false,
    counter: 0,
});

fn does_a1(_state: &mut InternalShellState, _args: &[String]) -> Result<String, ShiError> {
    Ok("A1 is done".to_owned())
}

fn start_counter(_state: &mut InternalShellState, _args: &[String]) -> Result<String, ShiError> {
    G_EXT_SHELL_STATE.lock().unwrap().counting = true;

    Ok("Counter on".to_string())
}

fn stop_counter(_state: &mut InternalShellState, _args: &[String]) -> Result<String, ShiError> {
    G_EXT_SHELL_STATE.lock().unwrap().counting = false;

    Ok("Counter off".to_string())
}

fn on_shell_update(_frame: usize) -> Option<()> {
    if G_EXT_SHELL_STATE.lock().unwrap().counting {
        let counter = G_EXT_SHELL_STATE.lock().unwrap().counter;
        info!("Counter: {counter}");
        G_EXT_SHELL_STATE.lock().unwrap().counter += 1;
    }

    Some(())
}

fn main() {
    drivers::logging::init_logging_with_level(log::LevelFilter::Trace);

    drivers::shell::spawn_shell_loop_thread(
        || InternalShellState {},
        || {
            vec![
                parent!("A", cmd!("1", "Does A1", does_a1)),
                parent!(
                    "counter",
                    cmd!("on", "Start the counter", start_counter),
                    cmd!("off", "Stop the counter", stop_counter),
                ),
            ]
        },
        on_shell_update,
    )
    .join()
    .unwrap()
    .unwrap();
}
