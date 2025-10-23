use crc32fast::Hasher;
use std::{hash::Hash, sync::Mutex};

use credit_store_demo::{
    autogen::schema::ObjState,
    db, drivers,
    macros::diesel_hist_models::{CreateSpanFrameError, SpanFrame},
};
use deterministic_hash::DeterministicHasher;
use diesel::{RunQueryDsl, SqliteConnection, query_dsl::methods::FilterDsl};
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

    let ev = coin_store::insert_event_for_obj(
        &mut mut_state.conn,
        obj_id as i32,
        &mut_state.cur_span_frame,
        ObjState::Insert,
        "create user",
        new_common,
    )
    .map_err(|e| ShiError::General { msg: e.to_string() })?;

    // Update partial for this new event
    sync_coin_store_events_partial(&mut mut_state.conn, ev.id)
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

    let ev = coin_store::insert_event_for_obj(
        &mut mut_state.conn,
        obj_id as i32,
        &mut_state.cur_span_frame,
        ObjState::Delete,
        "create user",
        new_common,
    )
    .map_err(|e| ShiError::General { msg: e.to_string() })?;

    // Update partial for this new event
    sync_coin_store_events_partial(&mut mut_state.conn, ev.id)
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

    b.push_record(["created_on", "person", "total_coins", "description"]);

    for (timestamp, person, coins, description) in table_to_print {
        b.push_record([timestamp, person, coins, description]);
    }

    let mut table = b.build();

    table.with(Style::modern_rounded());

    table.to_string()
}
pub fn display_pretty_table_for_records_toggled(
    table_to_print: &[(String, String, String, String, String, String)],
) -> String {
    use tabled::{builder::Builder, settings::Style};

    let mut b = Builder::with_capacity(3, 0);

    b.push_record([
        "id",
        "toggled",
        "created_on",
        "person",
        "total_coins",
        "description",
    ]);

    for (toggled, id, timestamp, person, coins, description) in table_to_print {
        b.push_record([toggled, id, timestamp, person, coins, description]);
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
        "(span: {}, frame: {})\n{}",
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
        "(span: {}, frame: {})\n{}",
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
        "(span: {}, frame: {})\n{}",
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
        "(span: {}, frame: {})\n{}",
        mut_state.cur_span_frame.span,
        mut_state.cur_span_frame.frame,
        display_pretty_table_for_records(&table_to_print)
    ))
}

fn set_coin_store_events_partial_to_full(
    conn: &mut SqliteConnection,
) -> Result<(), diesel::result::Error> {
    use credit_store_demo::autogen::schema::coin_store_events_grouped::dsl;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let objects: Vec<coin_store::EventGrouped> = dsl::coin_store_events_grouped
        .select(coin_store::EventGrouped::as_select())
        .get_results(conn)?;

    coin_store::set_events_grouped_partial(conn, &objects)?;

    Ok(())
}

fn sync_coin_store_events_partial(
    conn: &mut SqliteConnection,
    new_event_id: i32,
) -> Result<(), diesel::result::Error> {
    use credit_store_demo::autogen::schema::coin_store_events_grouped::dsl;
    use credit_store_demo::autogen::schema::coin_store_events_grouped_partial::dsl as dsl_p;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let objects_p: Vec<coin_store::EventGroupedPartial> = dsl_p::coin_store_events_grouped_partial
        .select(coin_store::EventGroupedPartial::as_select())
        .get_results(conn)?;

    let objects: Vec<coin_store::EventGrouped> = dsl::coin_store_events_grouped
        .select(coin_store::EventGrouped::as_select())
        .get_results(conn)?;

    let new_objects = objects
        .into_iter()
        .filter(|object| {
            let in_partial = objects_p
                .iter()
                .any(|object_p| object_p.ev_id == object.ev_id);

            let added_now = object.ev_id == new_event_id;

            in_partial || added_now
        })
        .collect::<Vec<_>>();

    coin_store::set_events_grouped_partial(conn, &new_objects)?;

    Ok(())
}

fn set_coin_store_events_partial_to_full_if_empty(
    conn: &mut SqliteConnection,
) -> Result<(), diesel::result::Error> {
    use credit_store_demo::autogen::schema::coin_store_events_grouped_partial::dsl as dsl_p;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let objects_p: Vec<coin_store::EventGroupedPartial> = dsl_p::coin_store_events_grouped_partial
        .select(coin_store::EventGroupedPartial::as_select())
        .get_results(conn)?;

    if !objects_p.is_empty() {
        return Ok(());
    }

    set_coin_store_events_partial_to_full(conn)?;

    Ok(())
}

fn coin_store_toggle_by_id(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::autogen::schema::coin_store_diffs::dsl as dsl_d;
    use credit_store_demo::autogen::schema::coin_store_events::dsl as dsl_e;
    use credit_store_demo::autogen::schema::coin_store_events_grouped::dsl;
    use credit_store_demo::autogen::schema::coin_store_events_grouped_partial::dsl as dsl_p;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let objects_p: Vec<coin_store::EventGroupedPartial> = dsl_p::coin_store_events_grouped_partial
        .select(coin_store::EventGroupedPartial::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let objects: Vec<coin_store::EventGrouped> = dsl::coin_store_events_grouped
        .select(coin_store::EventGrouped::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let events_joined: Vec<(coin_store::Event, coin_store::Diff)> = dsl_e::coin_store_events
        .inner_join(dsl_d::coin_store_diffs)
        .select((
            coin_store::Event::as_select(),
            coin_store::Diff::as_select(),
        ))
        .load::<(coin_store::Event, coin_store::Diff)>(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let events_joined_enabled: Vec<(String, &coin_store::Event, &coin_store::Diff)> = events_joined
        .iter()
        .map(|(event, diff)| {
            let in_partial = objects_p.iter().any(|object_p| object_p.ev_id == event.id);

            if in_partial {
                ("true".to_owned(), event, diff)
            } else {
                ("false".to_owned(), event, diff)
            }
        })
        .collect::<Vec<_>>();

    let table_to_print = events_joined_enabled
        .iter()
        .map(|(en, row_ev, row_diff)| {
            (
                format!("{}", row_ev.id),
                en.clone(),
                display_timestamp(row_ev.created_on_ts),
                row_diff.person.to_inner(),
                format!("{}", row_diff.coins),
                row_ev.ev_desc.clone(),
            )
        })
        .collect::<Vec<_>>();

    println!(
        "{}",
        display_pretty_table_for_records_toggled(&table_to_print)
    );

    let ev_id_toggled: u32 = match drivers::read_input_from_user_until_valid_or_quit(
        "Select event id to toggle (u32)",
    ) {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let new_objects = objects
        .into_iter()
        .filter(|object| {
            let in_partial = objects_p
                .iter()
                .any(|object_p| object_p.ev_id == object.ev_id);

            let toggled_now = object.ev_id == (ev_id_toggled as i32);

            in_partial ^ toggled_now
        })
        .collect::<Vec<_>>();

    coin_store::set_events_grouped_partial(&mut mut_state.conn, &new_objects)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    Ok("Toggle applied".to_string())
}

fn coin_store_undo_toggle_by_desc(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::autogen::schema::coin_store_diffs::dsl as dsl_d;
    use credit_store_demo::autogen::schema::coin_store_events::dsl as dsl_e;
    use credit_store_demo::autogen::schema::coin_store_events_grouped::dsl;
    use credit_store_demo::autogen::schema::coin_store_events_grouped_partial::dsl as dsl_p;
    use credit_store_demo::db::models::*;
    use diesel::prelude::*;

    let objects_p: Vec<coin_store::EventGroupedPartial> = dsl_p::coin_store_events_grouped_partial
        .select(coin_store::EventGroupedPartial::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let objects: Vec<coin_store::EventGrouped> = dsl::coin_store_events_grouped
        .select(coin_store::EventGrouped::as_select())
        .get_results(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let events_joined: Vec<(coin_store::Event, coin_store::Diff)> = dsl_e::coin_store_events
        .inner_join(dsl_d::coin_store_diffs)
        .select((
            coin_store::Event::as_select(),
            coin_store::Diff::as_select(),
        ))
        .load::<(coin_store::Event, coin_store::Diff)>(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let events_joined_enabled: Vec<(String, &coin_store::Event, &coin_store::Diff)> = events_joined
        .iter()
        .map(|(event, diff)| {
            let in_partial = objects_p.iter().any(|object_p| object_p.ev_id == event.id);

            if in_partial {
                ("true".to_owned(), event, diff)
            } else {
                ("false".to_owned(), event, diff)
            }
        })
        .collect::<Vec<_>>();

    let table_to_print = events_joined_enabled
        .iter()
        .map(|(en, row_ev, row_diff)| {
            (
                format!("{}", row_ev.id),
                en.clone(),
                display_timestamp(row_ev.created_on_ts),
                row_diff.person.to_inner(),
                format!("{}", row_diff.coins),
                row_ev.ev_desc.clone(),
            )
        })
        .collect::<Vec<_>>();

    println!(
        "{}",
        display_pretty_table_for_records_toggled(&table_to_print)
    );

    let desc_to_filter = match drivers::read_str_or_quit("Description substring") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let new_objects = objects
        .into_iter()
        .filter(|object| {
            let in_partial = objects_p
                .iter()
                .any(|object_p| object_p.ev_id == object.ev_id);

            let toggled_now = object.ev_desc.contains(&desc_to_filter);

            in_partial && !toggled_now || !in_partial && toggled_now
        })
        .collect::<Vec<_>>();

    coin_store::set_events_grouped_partial(&mut mut_state.conn, &new_objects)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    Ok("Toggle applied".to_string())
}

fn coin_store_span_push(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::db::models::*;

    let span_frames = coin_store::get_created_span_frames(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let opt_latest_upper_span_frame = span_frames
        .iter()
        .filter(|sf| sf.span == mut_state.cur_span_frame.span + 1)
        .max_by_key(|k| k.frame.abs());

    match opt_latest_upper_span_frame {
        Some(latest_upper_span_frame) => {
            let sf = coin_store::create_span_frame(
                &mut mut_state.conn,
                mut_state.cur_span_frame.span + 1,
                latest_upper_span_frame.frame + 1,
                "push new spanframe",
            )
            .map_err(|e| ShiError::General { msg: e.to_string() })?;
            mut_state.cur_span_frame = sf;
        }
        None => {
            let sf = coin_store::create_span_frame(
                &mut mut_state.conn,
                mut_state.cur_span_frame.span + 1,
                1,
                "push new spanframe",
            )
            .map_err(|e| ShiError::General { msg: e.to_string() })?;
            mut_state.cur_span_frame = sf;
        }
    }

    Ok(format!(
        "Pushed frame (span: {}, frame: {})",
        mut_state.cur_span_frame.span, mut_state.cur_span_frame.frame
    ))
}

fn coin_store_span_pop(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::db::models::*;

    let span_frames = coin_store::get_created_span_frames(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    // Go to the latest lower span frame
    if mut_state.cur_span_frame.span == 1 {
        return Ok("This is the lowest span".to_owned());
    }

    let opt_latest_lower_span_frame = span_frames
        .iter()
        .filter(|sf| sf.span == mut_state.cur_span_frame.span - 1)
        .max_by_key(|k| k.frame.abs());

    match opt_latest_lower_span_frame {
        Some(latest_lower_span_frame) => {
            mut_state.cur_span_frame = latest_lower_span_frame.clone();
            Ok(format!(
                "Popped to frame (span: {}, frame: {})",
                latest_lower_span_frame.span, latest_lower_span_frame.frame
            ))
        }
        None => Ok("Error: Could not find latest lower span frame".to_owned()),
    }
}

fn coin_store_switch(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::db::models::*;

    let span_frames = coin_store::get_created_span_frames(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let span: u32 = match drivers::read_input_from_user_until_valid_or_quit("span (u32)") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let span_exists = span_frames.iter().any(|sf| sf.span == span as i32);

    if !span_exists {
        return Ok("Error: span does not exist".to_owned());
    }

    let frame: u32 = match drivers::read_input_from_user_until_valid_or_quit("frame (u32)") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    let opt_sf = span_frames
        .iter()
        .find(|sf| sf.span == span as i32 && sf.frame == frame as i32);

    match opt_sf {
        Some(sf) => {
            mut_state.cur_span_frame = sf.clone();
            Ok(format!(
                "Switched to frame (span: {}, frame: {})",
                sf.span, sf.frame
            ))
        }
        None => Ok("Error: No such span frame found".to_owned()),
    }
}

fn coin_store_soft_reset(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::db::models::*;

    let span_frames = coin_store::get_created_span_frames(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let opt_latest_sf_in_span = span_frames
        .iter()
        .filter(|sf| sf.span == mut_state.cur_span_frame.span)
        .max_by_key(|k| k.frame.abs());

    match opt_latest_sf_in_span {
        Some(sf) => {
            let new_sf = coin_store::create_span_frame(
                &mut mut_state.conn,
                sf.span,
                sf.frame + 1,
                "soft reset",
            )
            .map_err(|e| ShiError::General { msg: e.to_string() })?;

            mut_state.cur_span_frame = new_sf.clone();

            Ok(format!(
                "Switched to frame (span: {}, frame: {})",
                new_sf.span, new_sf.frame
            ))
        }
        None => Ok("Error: Could not find latest span frame".to_owned()),
    }
}

fn coin_store_hard_reset(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::db::models::*;

    let span_frames = coin_store::get_created_span_frames(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let opt_latest_sf_in_span = span_frames
        .iter()
        .filter(|sf| sf.span == 1)
        .max_by_key(|k| k.frame.abs());

    match opt_latest_sf_in_span {
        Some(sf) => {
            let new_sf = coin_store::create_span_frame(
                &mut mut_state.conn,
                sf.span,
                sf.frame + 1,
                "hard reset",
            )
            .map_err(|e| ShiError::General { msg: e.to_string() })?;

            mut_state.cur_span_frame = new_sf.clone();

            Ok(format!(
                "Switched to frame (span: {}, frame: {})",
                new_sf.span, new_sf.frame
            ))
        }
        None => Ok("Error: Could not find latest span frame".to_owned()),
    }
}

fn coin_store_actually_reset(
    mut_state: &mut InternalShellState,
    _args: &[String],
) -> Result<String, ShiError> {
    use credit_store_demo::autogen::schema::coin_store_diffs::dsl as dsl_d;
    use credit_store_demo::autogen::schema::coin_store_events::dsl;

    let del_resp = match drivers::read_str_or_quit("Really delete everything? (yes/any)") {
        Some(item) => item,
        None => return Ok("".to_owned()),
    };

    if del_resp == "yes" {
        diesel::delete(dsl_d::coin_store_diffs)
            .execute(&mut mut_state.conn)
            .map_err(|e| ShiError::General { msg: e.to_string() })?;

        diesel::delete(dsl::coin_store_events)
            .execute(&mut mut_state.conn)
            .map_err(|e| ShiError::General { msg: e.to_string() })?;

        set_coin_store_events_partial_to_full(&mut mut_state.conn)
            .map_err(|e| ShiError::General { msg: e.to_string() })?;

        let res_sf = get_or_create_init_span_frame(&mut mut_state.conn);

        match res_sf {
            Ok(sf) => {
                mut_state.cur_span_frame = sf.clone();
            }
            Err(e) => {
                return Ok(format!("Error: Failed to create spanframe: {}", e));
            }
        }

        Ok("deleted everything".to_owned())
    } else {
        Ok("Did nothing".to_owned())
    }
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

    let ev = coin_store::insert_event_for_obj(
        &mut mut_state.conn,
        obj_id as i32,
        &mut_state.cur_span_frame,
        ObjState::Update,
        &desc,
        new_common,
    )
    .map_err(|e| ShiError::General { msg: e.to_string() })?;

    // Update partial for this new event
    sync_coin_store_events_partial(&mut mut_state.conn, ev.id)
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

    let ev = coin_store::insert_event_for_obj(
        &mut mut_state.conn,
        obj_id as i32,
        &mut_state.cur_span_frame,
        ObjState::Update,
        &desc,
        new_common,
    )
    .map_err(|e| ShiError::General { msg: e.to_string() })?;

    // Update partial for this new event
    sync_coin_store_events_partial(&mut mut_state.conn, ev.id)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    Ok("Added income for user".to_owned())
}

fn coin_store_ls(mut_state: &mut InternalShellState, _args: &[String]) -> Result<String, ShiError> {
    use credit_store_demo::db::models::*;

    let span_frames = coin_store::get_created_span_frames(&mut mut_state.conn)
        .map_err(|e| ShiError::General { msg: e.to_string() })?;

    let output = {
        let mut mut_output = "".to_owned();

        for span_frame in span_frames {
            if span_frame.span == mut_state.cur_span_frame.span
                && span_frame.frame == mut_state.cur_span_frame.frame
            {
                mut_output += &format!(
                    "==> (span: {}, frame: {})\n",
                    span_frame.span, span_frame.frame
                );
            } else {
                mut_output +=
                    &format!("(span: {}, frame: {})\n", span_frame.span, span_frame.frame);
            }
        }

        mut_output
    };

    Ok(output)
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

    set_coin_store_events_partial_to_full_if_empty(&mut conn)
        .expect("Failed to set coin_store_events_partial to full");

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
                                "Show the current coin amounts for all users in current span/frame (accounting for toggle)",
                                coin_store_show_partial_wallet,
                            ),
                            cmd!(
                                "records",
                                "Show the current span/frame records of transactions made (accounting for toggle)",
                                coin_store_show_partial_records,
                            ),
                        )
                    ),
                    parent!(
                        "toggle",
                        cmd!(
                            "id",
                            "Toggles whether a transaction is enabled by id",
                            coin_store_toggle_by_id,
                        ),
                        cmd!(
                            "desc",
                            "Toggles whether a transaction is enabled by description substring",
                            coin_store_undo_toggle_by_desc,
                        ),
                    ),
                    cmd!(
                        "ls",
                        "List the span/frame tree and the user's curent position within it",
                        coin_store_ls,
                    ),
                    parent!(
                        "span",
                        cmd!(
                            "push",
                            "Extends transactions over to a frame at an upper span",
                            coin_store_span_push,
                        ),
                        cmd!(
                            "pop",
                            "Extends transactions over to a frame at an upper span",
                            coin_store_span_pop,
                        ),
                    ),
                    cmd!("switch", "Switch to a given span frame", coin_store_switch,),
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
                        cmd!(
                            "actually",
                            "Actually deletes all data",
                            coin_store_actually_reset,
                        ),
                    )
                ),
            ]
        },
        on_shell_update,
    );

    shell_join.join().unwrap().unwrap();
}
