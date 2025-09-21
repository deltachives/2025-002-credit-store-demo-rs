use std::{
    fmt::Display,
    sync::mpsc::{Receiver, RecvError, SendError, Sender},
};

use crate::db::{
    event_accumulator::{EaCommand, EaWorkError, EventSourcable, EventSourcableError},
    models::*,
};

use diesel::prelude::*;

use thiserror::Error;

use chrono;

use log::*;

#[derive(Debug, strum::VariantArray)]
pub enum HeadTable {
    CreditStore,
}

#[derive(Error, Debug)]
pub enum DbActionAssertionError {
    #[error("Services wrong table. Expected {expected:?} got {actual:?}")]
    InvalidTableServiced { expected: String, actual: String },
}

#[derive(Error, Debug)]
pub enum DbActionError {
    #[error("Table action error: {0:?}")]
    DieselError(#[from] diesel::result::Error),

    #[error("Failed to send message: {0:?}")]
    SendError(#[from] SendError<EaCommand>),

    #[error("Failed to receive message: {0:?}")]
    RecvError(#[from] RecvError),

    #[error("Error from event accumulator for work: {0:?}")]
    EaWorkError(#[from] EaWorkError),

    #[error("Assertion failed: {0:?}")]
    AssertionFailed(#[from] DbActionAssertionError),
}

impl Display for HeadTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeadTable::CreditStore => write!(f, "credit_store_head"),
        }
    }
}

pub fn send_write_event_for_head(
    tx_ea_msg: &Sender<EaCommand>,
    head_table: HeadTable,
) -> Result<(), SendError<EaCommand>> {
    let table_name = head_table.to_string();

    tx_ea_msg.send(EaCommand::DbWrite { table_name })?;

    Ok(())
}

/// Inserts an event creating a new object
pub fn credit_store_event_blocking_insert(
    mut_conn: &mut SqliteConnection,
    tx_cmd_to_ea: &Sender<EaCommand>,
    rx_ea_work_done: &Receiver<Result<String, EaWorkError>>,
    object: &CreditStoreObject,
    event_stack_level: i32,
) -> Result<CreditStoreEvent, DbActionError> {
    use crate::autogen::schema::credit_store_events::dsl;

    // Since we're inserting a new object, it should get a unique id.
    // We want each person to be unique, so let's hash their name as an ID.
    let new_object_id = object.person.as_bytes().iter().map(|n| i32::from(*n)).sum();

    let new_event = NewCreditStoreEvent {
        person: object.person.clone(),
        credits: object.credits,
        opt_object_id: Some(new_object_id),
        opt_event_id: None,
        opt_event_arg: None,
        event_stack_level,
        event_action: crate::autogen::schema::EventAction::Insert,
        created_on: &format!("{:?}", chrono::offset::Local::now()),
    };

    let entry: CreditStoreEvent = diesel::insert_into(dsl::credit_store_events)
        .values(new_event)
        .returning(CreditStoreEvent::as_returning())
        .get_result(mut_conn)?;

    send_write_event_for_head(tx_cmd_to_ea, HeadTable::CreditStore)?;

    info!("awaiting event accumulator");

    let table_serviced = rx_ea_work_done.recv()??;

    info!("Table {table_serviced} has been serviced!");

    if table_serviced != HeadTable::CreditStore.to_string() {
        // We are the only ones who own the receiver, so no service race condition should happen.
        Err(DbActionError::AssertionFailed(
            DbActionAssertionError::InvalidTableServiced {
                expected: HeadTable::CreditStore.to_string(),
                actual: table_serviced,
            },
        ))
    } else {
        Ok(entry)
    }
}

pub struct CreditStoreEventSourcable {}

impl crate::db::event_accumulator::ReadEventTableVersion for CreditStoreEventSourcable {
    fn read_event_table_version(
        &self,
        mut_conn: &mut SqliteConnection,
    ) -> Result<Option<i32>, EventSourcableError> {
        use crate::autogen::schema::credit_store_version::dsl;

        let opt_version_entry: Option<CreditStoreVersion> = dsl::credit_store_version
            .find(1)
            .select(CreditStoreVersion::as_select())
            .first(mut_conn)
            .optional()?;

        match opt_version_entry {
            Some(version_entry) => Ok(version_entry.opt_event_id),
            None => {
                // Create the row
                let entry: CreditStoreVersion = diesel::insert_into(dsl::credit_store_version)
                    .values(&NewCreditStoreVersion { opt_event_id: None })
                    .returning(CreditStoreVersion::as_returning())
                    .get_result(mut_conn)?;

                if entry.opt_event_id.is_some() {
                    Err(EventSourcableError::AssertionFailed(format!(
                        "opt_event_id is not none: {entry:?}"
                    )))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl crate::db::event_accumulator::WriteEventTableVersion for CreditStoreEventSourcable {
    fn write_event_table_version(
        &self,
        mut_conn: &mut SqliteConnection,
        opt_event_id: Option<i32>,
    ) -> Result<(), EventSourcableError> {
        use crate::autogen::schema::credit_store_version::dsl;

        let entry: CreditStoreVersion = diesel::update(dsl::credit_store_version.find(1))
            .set(dsl::opt_event_id.eq(opt_event_id))
            .returning(CreditStoreVersion::as_returning())
            .get_result(mut_conn)?;

        if entry.opt_event_id != opt_event_id {
            Err(EventSourcableError::AssertionFailed(format!(
                "opt_event_id is not none: {entry:?}"
            )))
        } else {
            Ok(())
        }
    }
}

impl crate::db::event_accumulator::HasEventObject for CreditStoreEventSourcable {
    type Object = CreditStoreObject;
}

impl crate::db::event_accumulator::ReadEventTable for CreditStoreEventSourcable {
    fn read_event_table(
        &self,
        mut_conn: &mut SqliteConnection,
        opt_from_event_id: Option<i32>,
    ) -> Result<Vec<(crate::db::event_accumulator::EventData, Self::Object)>, EventSourcableError>
    {
        use crate::autogen::schema::credit_store_events::dsl;

        let entries: Vec<CreditStoreEvent> = match opt_from_event_id {
            Some(from_event_id) => dsl::credit_store_events
                .filter(dsl::id.gt(from_event_id))
                .select(CreditStoreEvent::as_select())
                .load(mut_conn)?,
            None => dsl::credit_store_events
                .select(CreditStoreEvent::as_select())
                .load(mut_conn)?,
        };

        let decomposed = entries
            .into_iter()
            .map(|entry| {
                (
                    crate::db::event_accumulator::EventData {
                        id: entry.id,
                        opt_object_id: entry.opt_object_id,
                        opt_event_id: entry.opt_event_id,
                        opt_event_arg: entry.opt_event_arg,
                        event_stack_level: entry.event_stack_level,
                        event_action: entry.event_action,
                        created_on: entry.created_on,
                    },
                    Self::Object {
                        person: entry.person,
                        credits: entry.credits,
                    },
                )
            })
            .collect::<Vec<_>>();

        Ok(decomposed)
    }
}

impl crate::db::event_accumulator::InsertIntoHeadTable for CreditStoreEventSourcable {
    fn insert_into_head_table(
        &self,
        mut_conn: &mut SqliteConnection,
        objects: &[Self::Object],
    ) -> Result<(), EventSourcableError> {
        use crate::autogen::schema::credit_store_head::dsl;

        let new_records = objects
            .iter()
            .map(|obj| NewCreditStoreRec {
                person: &obj.person,
                credits: obj.credits,
            })
            .collect::<Vec<_>>();

        let num_created: usize = diesel::insert_into(dsl::credit_store_head)
            .values(&new_records)
            .execute(mut_conn)?;

        if num_created != new_records.len() {
            Err(EventSourcableError::AssertionFailed(format!(
                "Expected to write {num_created} but wrote {}",
                new_records.len()
            )))
        } else {
            Ok(())
        }
    }
}

impl crate::db::event_accumulator::UpdateHeadTableRow for CreditStoreEventSourcable {
    fn update_head_table_row(
        &self,
        mut_conn: &mut SqliteConnection,
        object_id: i32,
        object: Self::Object,
    ) -> Result<(), EventSourcableError> {
        use crate::autogen::schema::credit_store_head::dsl;

        let entry: CreditStoreHeadRec = diesel::update(dsl::credit_store_head.find(object_id))
            .set(NewCreditStoreRec {
                person: &object.person,
                credits: object.credits,
            })
            .returning(CreditStoreHeadRec::as_returning())
            .get_result(mut_conn)?;

        if entry.id != object_id {
            Err(EventSourcableError::AssertionFailed(format!(
                "Expected entry id {} to be identical to object_id {}",
                entry.id, object_id
            )))
        } else {
            Ok(())
        }
    }
}

impl crate::db::event_accumulator::DeleteHeadTableRow for CreditStoreEventSourcable {
    fn delete_head_table_row(
        &self,
        mut_conn: &mut SqliteConnection,
        object_id: i32,
    ) -> Result<(), EventSourcableError> {
        use crate::autogen::schema::credit_store_head::dsl;

        let _ = diesel::delete(dsl::credit_store_head.find(object_id))
            .returning(CreditStoreHeadRec::as_returning())
            .execute(mut_conn)?;

        Ok(())
    }
}

impl crate::db::event_accumulator::ClearHeadTable for CreditStoreEventSourcable {
    fn clear_head_table(&self, mut_conn: &mut SqliteConnection) -> Result<(), EventSourcableError> {
        use crate::autogen::schema::credit_store_head::dsl;

        let _ = diesel::delete(dsl::credit_store_head).execute(mut_conn)?;

        Ok(())
    }
}

impl crate::db::event_accumulator::SourceHeadTable for CreditStoreEventSourcable {
    fn source_head_table_row(
        &self,
        mut_conn: &mut SqliteConnection,
    ) -> Result<(), EventSourcableError> {
        use crate::autogen::schema::credit_store::dsl as dsl2;
        use crate::autogen::schema::credit_store_head::dsl;

        // Clear head table
        let _ = diesel::delete(dsl::credit_store_head).execute(mut_conn)?;

        // Read base table
        let entries: Vec<CreditStoreRec> = dsl2::credit_store
            .select(CreditStoreRec::as_select())
            .load(mut_conn)?;

        // Insert its contents to the head table
        let new_records = entries
            .iter()
            .map(|obj| NewCreditStoreRec {
                person: &obj.person,
                credits: obj.credits,
            })
            .collect::<Vec<_>>();

        let _ = diesel::insert_into(dsl::credit_store_head)
            .values(new_records)
            .execute(mut_conn)?;

        Ok(())
    }
}

impl EventSourcable for CreditStoreEventSourcable {}
