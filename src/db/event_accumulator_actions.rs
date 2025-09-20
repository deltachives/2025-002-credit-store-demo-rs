use std::{
    fmt::Display,
    sync::mpsc::{SendError, Sender},
};

use crate::db::{
    event_accumulator::{EaThreadMessage, EventSourcable, EventSourcableError},
    models::*,
};

use diesel::prelude::*;

// use diesel::{
//     associations::HasTable,
//     query_builder::{AsQuery, IntoUpdateTarget, QueryFragment, UpdateStatement},
//     sqlite::Sqlite,
// };

use thiserror::Error;

#[derive(Error, Debug)]
pub enum HeadRecOpError {
    #[error("Table action error: {0:?}")]
    DieselError(#[from] diesel::result::Error),

    #[error("Failed to send message: {0:?}")]
    ThreadMessageSendError(#[from] SendError<EaThreadMessage>),
}

// pub trait InsertHeadRec {
//     type NewRec;
//     type Rec;

//     fn insert_head_rec(
//         conn: &mut SqliteConnection,
//         table: impl Table,
//         object: Self::NewRec,
//         tx_ea_msg: Sender<EaThreadMessage>,
//     ) -> Result<Self::Rec, HeadRecOpError> {
//         let out = diesel::insert_into(table)
//             .values(&object)
//             .returning(CreditStoreHeadRec::as_returning())
//             .get_result(conn)?;

//         tx_ea_msg.send(EaThreadMessage::DbWrite {
//             table_name: object.get_table_group_id(),
//         })?;

//         Ok(out)
//     }
// }

// // A very complex automation... Using macro rules or derives or other methods might be better
// pub trait UpdateHeadRec {
//     fn update_head_rec<Model, Tab>(
//         conn: &mut SqliteConnection,
//         table: Tab,
//         object: Model,
//         tx_ea_msg: Sender<EaThreadMessage>,
//     ) -> Result<usize, HeadRecOpError>
//     where
//         Model: AsChangeset<Target = Tab> + Insertable<Tab>,
//         Tab: Identifiable
//             + QueryFragment<Sqlite>
//             + HasTable<Table = Tab>
//             + diesel::Table
//             + IntoUpdateTarget
//             + AsChangeset
//             + GetTableName,
//         <Tab as QuerySource>::FromClause: QueryFragment<Sqlite>,
//         <Tab as IntoUpdateTarget>::WhereClause: QueryFragment<Sqlite>,
//         <Model as AsChangeset>::Changeset: QueryFragment<Sqlite>,
//         UpdateStatement<
//             Tab,
//             <Tab as IntoUpdateTarget>::WhereClause,
//             <Model as AsChangeset>::Changeset,
//         >: AsQuery,
//     {
//         let written = diesel::update(table).set(object).execute(conn)?;

//         tx_ea_msg.send(EaThreadMessage::DbWrite {
//             table_name: table.get_table_name(),
//         })?;

//         Ok(written)
//     }
// }

#[derive(Debug, strum::VariantArray)]
pub enum HeadTable {
    CreditStore,
}

impl Display for HeadTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeadTable::CreditStore => write!(f, "credit_store_head"),
        }
    }
}

pub fn send_write_event_for_head(
    tx_ea_msg: &Sender<EaThreadMessage>,
    head_table: HeadTable,
) -> Result<(), SendError<EaThreadMessage>> {
    let table_name = head_table.to_string();

    tx_ea_msg.send(EaThreadMessage::DbWrite { table_name })?;

    Ok(())
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
