use crc32fast::Hasher;
use std::{hash::Hash, sync::Mutex};

use credit_store_demo::{
    autogen::schema::ObjState,
    db, drivers,
    macros::diesel_hist_models::{CreateSpanFrameError, SpanFrame},
};
use deterministic_hash::DeterministicHasher;
use diesel::{SqliteConnection, query_dsl::methods::FilterDsl};
use log::*;
use shi::{cmd, error::ShiError, parent};
use tap::prelude::*;

struct InternalShellState {
    conn: SqliteConnection,
    cur_span_frame: SpanFrame,
}

struct _ExternalShellState {}

static _G_EXT_SHELL_STATE: Mutex<_ExternalShellState> = Mutex::new(_ExternalShellState {});

fn on_shell_update(_frame: usize) -> Option<()> {
    Some(())
}

fn show_version(_mut_state: &mut InternalShellState, _args: &[String]) -> Result<String, ShiError> {
    Ok("v1.0.0".to_owned())
}

/// Add a new user to the current coin store frame with 0 coins
fn coin_store_add_user(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::autogen::schema::coin_store_events_grouped::dsl;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let person: Person =
        match drivers::read_input_from_user_until_valid_or_quit("person (NOT admin!)") {
            Some(item) => item,
            None => return Ok("".to_owned()),
        };

    // let coins: i32 = match drivers::read_input_from_user_until_valid_or_quit("coins (i32)") {
    //     Some(item) => item,
    //     None => return Ok("".to_owned()),
    // };

    // Check if the user already exists in the current spanframe
    let results: Vec<coin_store::EventGrouped> = dsl::coin_store_events_grouped
        .pipe(|tbl| FilterDsl::filter(tbl, dsl::person.eq(&person)))
        .select(coin_store::EventGrouped::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    if !results.is_empty() {
        return Ok("Error: User already exists".to_owned());
    }

    let new_common = coin_store::NewCommon {
        coins: 0,
        person: &person,
    };

    let obj_id = {
        let mut hasher = DeterministicHasher::new(Hasher::new());
        person.hash(&mut hasher);

        hasher.as_inner().clone().finalize()
    };

    let _ = coin_store::insert_event_for_obj(
        &mut mut_state.conn,
        obj_id as i32,
        &mut_state.cur_span_frame,
        ObjState::Insert,
        "create user",
        new_common,
    )
    .map_err(|e| ShiError::General { msg: e.to_string() })?;

    Ok("Created user".to_owned())
}

fn coin_store_delete_user(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::autogen::schema::coin_store_events_grouped::dsl;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let person: Person =
        match drivers::read_input_from_user_until_valid_or_quit("person (NOT admin!)") {
            Some(item) => item,
            None => return Ok("".to_owned()),
        };

    // Check if the user already exists in the current spanframe
    let results: Vec<coin_store::EventGrouped> = dsl::coin_store_events_grouped
        .pipe(|tbl| FilterDsl::filter(tbl, dsl::person.eq(&person)))
        .select(coin_store::EventGrouped::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    if results.is_empty() {
        return Ok("Error: User does not exist".to_owned());
    }

    let new_common = coin_store::NewCommon {
        coins: 0,
        person: &person,
    };

    let obj_id = {
        let mut hasher = DeterministicHasher::new(Hasher::new());
        person.hash(&mut hasher);

        hasher.as_inner().clone().finalize()
    };

    let _ = coin_store::insert_event_for_obj(
        &mut mut_state.conn,
        obj_id as i32,
        &mut_state.cur_span_frame,
        ObjState::Delete,
        "create user",
        new_common,
    )
    .map_err(|e| ShiError::General { msg: e.to_string() })?;

    Ok("Deleted user".to_owned())
}

pub fn display_pretty_table(table_to_print: &[(String, String)]) -> String {
    use tabled::{builder::Builder, settings::Style};

    let mut b = Builder::with_capacity(3, 0);

    b.push_record(["person", "total_coins"]);

    for (person, coins) in table_to_print {
        b.push_record([person, coins]);
    }

    let mut table = b.build();

    table.with(Style::modern_rounded());

    table.to_string()
}

pub fn display_pretty_table_for_records(
    table_to_print: &[(String, String, String, String)],
) -> String {
    use tabled::{builder::Builder, settings::Style};

    let mut b = Builder::with_capacity(3, 0);

    b.push_record(["timestamp", "person", "total_coins", "description"]);

    for (timestamp, person, coins, description) in table_to_print {
        b.push_record([timestamp, person, coins, description]);
    }

    let mut table = b.build();

    table.with(Style::modern_rounded());

    table.to_string()
}

pub fn display_timestamp(timestamp: f32) -> String {
    use chrono::{DateTime, TimeZone, Utc};

    let timestamp_millis = timestamp as i64;

    let seconds = timestamp_millis / 1000;
    let nanoseconds = (timestamp_millis % 1000) * 1_000_000;

    let datetime: DateTime<Utc> = Utc
        .timestamp_opt(seconds, nanoseconds as u32)
        .single()
        .unwrap();

    format!("{}", datetime)
}

fn coin_store_show_wallet(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::autogen::schema::coin_store_hist::dsl;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let objects = dsl::coin_store_hist
        .pipe(|tbl| {
            FilterDsl::filter(
                tbl,
                dsl::grp_span
                    .eq(mut_state.cur_span_frame.span)
                    .and(dsl::grp_frame.eq(mut_state.cur_span_frame.frame)),
            )
        })
        .select(coin_store::Hist::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let table_to_print = objects
        .iter()
        .map(|row| (row.person.to_inner(), format!("{}", row.coins)))
        .collect::<Vec<_>>();

    Ok(format!(
        "span: {}, frame: {}\n{}",
        mut_state.cur_span_frame.span,
        mut_state.cur_span_frame.frame,
        display_pretty_table(&table_to_print)
    ))
}

fn coin_store_show_partial_wallet(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::autogen::schema::coin_store_hist_partial::dsl;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let objects = dsl::coin_store_hist_partial
        .pipe(|tbl| {
            FilterDsl::filter(
                tbl,
                dsl::grp_span
                    .eq(mut_state.cur_span_frame.span)
                    .and(dsl::grp_frame.eq(mut_state.cur_span_frame.frame)),
            )
        })
        .select(coin_store::HistPartial::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let table_to_print = objects
        .iter()
        .map(|row| (row.person.to_inner(), format!("{}", row.coins)))
        .collect::<Vec<_>>();

    Ok(format!(
        "span: {}, frame: {}\n{}",
        mut_state.cur_span_frame.span,
        mut_state.cur_span_frame.frame,
        display_pretty_table(&table_to_print)
    ))
}

fn coin_store_show_records(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::autogen::schema::coin_store_events_grouped::dsl;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let objects: Vec<coin_store::EventGrouped> = dsl::coin_store_events_grouped
        .pipe(|tbl| {
            FilterDsl::filter(
                tbl,
                dsl::grp_span
                    .eq(mut_state.cur_span_frame.span)
                    .and(dsl::grp_frame.eq(mut_state.cur_span_frame.frame)),
            )
        })
        .select(coin_store::EventGrouped::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let table_to_print = objects
        .iter()
        .map(|row| {
            (
                display_timestamp(row.created_on_ts),
                row.person.to_inner(),
                format!("{}", row.coins),
                row.ev_desc.clone(),
            )
        })
        .collect::<Vec<_>>();

    Ok(format!(
        "span: {}, frame: {}\n{}",
        mut_state.cur_span_frame.span,
        mut_state.cur_span_frame.frame,
        display_pretty_table_for_records(&table_to_print)
    ))
}

fn coin_store_show_partial_records(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::autogen::schema::coin_store_events_grouped_partial::dsl;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let objects = dsl::coin_store_events_grouped_partial
        .pipe(|tbl| {
            FilterDsl::filter(
                tbl,
                dsl::grp_span
                    .eq(mut_state.cur_span_frame.span)
                    .and(dsl::grp_frame.eq(mut_state.cur_span_frame.frame)),
            )
        })
        .select(coin_store::EventGroupedPartial::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let table_to_print = objects
        .iter()
        .map(|row| {
            (
                display_timestamp(row.created_on_ts),
                row.person.to_inner(),
                format!("{}", row.coins),
                row.ev_desc.clone(),
            )
        })
        .collect::<Vec<_>>();

    Ok(format!(
        "span: {}, frame: {}\n{}",
        mut_state.cur_span_frame.span,
        mut_state.cur_span_frame.frame,
        display_pretty_table_for_records(&table_to_print)
    ))
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

fn coin_store_soft_reset(
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

fn coin_store_income(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::autogen::schema::coin_store_events_grouped::dsl;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let person: Person =
        match drivers::read_input_from_user_until_valid_or_quit("person (NOT admin!)") {
            Some(item) => item,
            None => return Ok("".to_owned()),
        };

    // Check if the user already exists in the current spanframe
    let results: Vec<coin_store::EventGrouped> = dsl::coin_store_events_grouped
        .pipe(|tbl| FilterDsl::filter(tbl, dsl::person.eq(&person)))
        .select(coin_store::EventGrouped::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    if results.is_empty() {
        return Ok("Error: User does not exist".to_owned());
    }

    let coins: u32 = match drivers::read_input_from_user_until_valid_or_quit("coins to add (u32)") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let desc = match drivers::read_str_or_quit("Description") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let new_common = coin_store::NewCommon {
        coins: coins as i32,
        person: &person,
    };

    let obj_id = {
        let mut hasher = DeterministicHasher::new(Hasher::new());
        person.hash(&mut hasher);

        hasher.as_inner().clone().finalize()
    };

    let _ = coin_store::insert_event_for_obj(
        &mut mut_state.conn,
        obj_id as i32,
        &mut_state.cur_span_frame,
        ObjState::Update,
        &desc,
        new_common,
    )
    .map_err(|e| ShiError::General { msg: e.to_string() })?;

    Ok("Added income for user".to_owned())
}

fn coin_store_expense(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::autogen::schema::coin_store_events_grouped::dsl;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let person: Person =
        match drivers::read_input_from_user_until_valid_or_quit("person (NOT admin!)") {
            Some(item) => item,
            None => return Ok("".to_owned()),
        };

    // Check if the user already exists in the current spanframe
    let results: Vec<coin_store::EventGrouped> = dsl::coin_store_events_grouped
        .pipe(|tbl| FilterDsl::filter(tbl, dsl::person.eq(&person)))
        .select(coin_store::EventGrouped::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    if results.is_empty() {
        return Ok("Error: User does not exist".to_owned());
    }

    let coins: u32 = match drivers::read_input_from_user_until_valid_or_quit("coins to add (u32)") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let desc = match drivers::read_str_or_quit("Description") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let new_common = coin_store::NewCommon {
        coins: -(coins as i32),
        person: &person,
    };

    let obj_id = {
        let mut hasher = DeterministicHasher::new(Hasher::new());
        person.hash(&mut hasher);

        hasher.as_inner().clone().finalize()
    };

    let _ = coin_store::insert_event_for_obj(
        &mut mut_state.conn,
        obj_id as i32,
        &mut_state.cur_span_frame,
        ObjState::Update,
        &desc,
        new_common,
    )
    .map_err(|e| ShiError::General { msg: e.to_string() })?;

    Ok("Added income for user".to_owned())
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

pub fn get_or_create_init_span_frame(
    conn: &mut SqliteConnection,
) -> Result<SpanFrame, CreateSpanFrameError> {
    // use credit_store_demo::autogen::schema::coin_store_events::dsl::*;
    use credit_store_demo::db::models::*;
    // use diesel::prelude::*;

    // let results: Vec<coin_store::Event> = coin_store_events
    //     .pipe(|tbl| FilterDsl::filter(tbl, ev_action.eq(EventAction::Open)))
    //     .select(coin_store::Event::as_select())
    //     .get_results(conn)
    //     .unwrap();

    let spanframes = coin_store::get_created_span_frames(conn)?;

    let opt_first_spanframe = spanframes.iter().find(|sf| sf.span == 1 && sf.frame == 1);

    match opt_first_spanframe {
        Some(first_spanframe) => Ok((*first_spanframe).clone()),
        None => Ok(coin_store::create_span_frame(conn, 1, 1, "First Frame!")?),
    }
}

fn main() {
    info!("Starting demo!");

    drivers::logging::init_logging_with_level(log::LevelFilter::Trace);

    let mut conn = db::loader::establish_connection().expect("Failed to initialize Sqlite db");

    let cur_span_frame =
        get_or_create_init_span_frame(&mut conn).expect("Failed to create first frame");

    let shell_join = drivers::shell::spawn_shell_loop_thread(
        || InternalShellState {
            conn,
            cur_span_frame,
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
                            "delete",
                            "Delete a user only within the current coin store frame",
                            coin_store_delete_user,
                        ),
                    ),
                    cmd!(
                        "income",
                        "Add income coins for a user in current span/frame",
                        coin_store_income,
                    ),
                    cmd!(
                        "expense",
                        "Add an expense order for a user in current span/frame",
                        coin_store_expense,
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
                        parent!(
                            "partial",
                            cmd!(
                                "wallet",
                                "Show the current coin amounts for all users in current span/frame (accounting for undos)",
                                coin_store_show_partial_wallet,
                            ),
                            cmd!(
                                "records",
                                "Show the current span/frame records of transactions made (accounting for undos)",
                                coin_store_show_partial_records,
                            ),
                        )
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
                    parent!(
                        "reset",
                        cmd!(
                            "soft",
                            "Resets back to previous span content in a new frame",
                            coin_store_soft_reset,
                        ),
                        cmd!(
                            "hard",
                            "Resets to a frame in the lowest span",
                            coin_store_hard_reset,
                        ),
                    )
                ),
            ]
        },
        on_shell_update,
    );

    shell_join.join().unwrap().unwrap();
}
