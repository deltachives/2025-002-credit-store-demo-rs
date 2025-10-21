use std::sync::Mutex;

use credit_store_demo::{
    autogen::schema::EventAction, db, drivers, macros::diesel_hist_models::SpanFrame,
};
use diesel::{SqliteConnection, query_dsl::methods::FilterDsl};
use log::*;
use shi::{cmd, error::ShiError, parent};
use tap::prelude::*;

struct InternalShellState {
    _conn: SqliteConnection,
    _cur_span_frame: SpanFrame,
}

struct _ExternalShellState {}

static _G_EXT_SHELL_STATE: Mutex<_ExternalShellState> = Mutex::new(_ExternalShellState {});

fn on_shell_update(_frame: usize) -> Option<()> {
    Some(())
}

fn show_version(_mut_state: &mut InternalShellState, _args: &[String]) -> Result<String, ShiError> {
    Ok("v1.0.0".to_owned())
}

fn coin_store_add_user(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_remove_user(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_delete_user(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_show_wallet(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_show_records(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_undo_toggle(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_undo_filter(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_branch(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_checkout(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_reset(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_hard_reset(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_record(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn coin_store_ls(
    _mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    todo!()
}

fn _credit_store_events_insert(
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

pub fn get_or_create_init_span_frame(conn: &mut SqliteConnection) -> SpanFrame {
    use credit_store_demo::autogen::schema::coin_store_events::dsl::*;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let _results: Vec<coin_store::Event> = coin_store_events
        .pipe(|tbl| FilterDsl::filter(tbl, ev_action.eq(EventAction::Open)))
        .select(coin_store::Event::as_select())
        .get_results(conn)
        .unwrap();

    todo!()
}

fn main() {
    info!("Starting demo!");

    drivers::logging::init_logging_with_level(log::LevelFilter::Trace);

    let mut conn = db::loader::establish_connection().expect("Failed to initialize Sqlite db");

    let cur_span_frame = get_or_create_init_span_frame(&mut conn);

    let shell_join = drivers::shell::spawn_shell_loop_thread(
        || InternalShellState {
            _conn: conn,
            _cur_span_frame: cur_span_frame,
        },
        || {
            vec![
                parent!(
                    "info",
                    cmd!("version", "Show current demo version", show_version,)
                ),
                parent!(
                    "coins",
                    parent!(
                        "users",
                        cmd!(
                            "add",
                            "Add a new user to the current coin store frame with 0 coins",
                            coin_store_add_user,
                        ),
                        cmd!(
                            "remove",
                            "Delete a user only within the current coin store frame",
                            coin_store_remove_user,
                        ),
                        cmd!(
                            "delete",
                            "Delete a user within all span frames",
                            coin_store_delete_user,
                        ),
                    ),
                    cmd!(
                        "record",
                        "Add a coin transaction record for a user in the current span/frame",
                        coin_store_record,
                    ),
                    parent!(
                        "show",
                        cmd!(
                            "wallet",
                            "Show the current coin amounts for all users in current span/frame",
                            coin_store_show_wallet,
                        ),
                        cmd!(
                            "records",
                            "Show the current span/frame records of transactions made",
                            coin_store_show_records,
                        ),
                    ),
                    parent!(
                        "undo",
                        cmd!(
                            "toggle",
                            "Toggles whether a transaction is enabled. Disabled transactions don't count towards wallet.",
                            coin_store_undo_toggle,
                        ),
                        cmd!(
                            "filter",
                            "Disables transactions by description",
                            coin_store_undo_filter,
                        ),
                    ),
                    cmd!(
                        "ls",
                        "List the span/frame tree and the user's curent position within it",
                        coin_store_ls,
                    ),
                    cmd!(
                        "branch",
                        "Carry transactions over to a new span frame",
                        coin_store_branch,
                    ),
                    cmd!(
                        "checkout",
                        "Checkout a current active frame",
                        coin_store_checkout,
                    ),
                    cmd!(
                        "reset",
                        "Resets back to previous span content in a new frame",
                        coin_store_reset,
                    ),
                    cmd!(
                        "hard_reset",
                        "Resets to a frame in the lowest span",
                        coin_store_hard_reset,
                    ),
                ),
            ]
        },
        on_shell_update,
    );

    shell_join.join().unwrap().unwrap();
}
