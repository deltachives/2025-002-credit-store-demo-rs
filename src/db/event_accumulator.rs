//! This module handles building virtual accumulated tables from events tables. It also handles
//! events tables being sourced.

use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::mpsc::{Receiver, RecvError, SendError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use diesel::SqliteConnection;
use log::*;
use thiserror::Error;

use crate::db::loader::{self, EstablishConnectionError};

pub enum EaThreadMessage {
    DbWrite {
        table_name: String,
    },

    /// On None, the event accumulator attempts to check out latest.
    DbCheckout {
        table_name: String,
        opt_event_id: Option<i32>,
    },

    Quit,
}

#[derive(Error, Debug)]
pub enum EventSourcableError {
    #[error("Got a diesel error for event sourcable: {0:?}")]
    DieselError(#[from] diesel::result::Error),

    #[error("Assertion failed for event sourcable: {0:?}")]
    AssertionFailed(String),

    #[error("Got an error for event sourcable: {0:?}")]
    Error(String),
}

pub trait ReadEventTableVersion {
    /// This should read the current version. If no row exists, it creates one and sets the current
    /// version to None.
    fn read_event_table_version(
        &self,
        mut_conn: &mut SqliteConnection,
    ) -> Result<Option<i32>, EventSourcableError>;
}

pub trait WriteEventTableVersion {
    /// This writes the new version. If no row exists, it creates it and sets it to that version.
    fn write_event_table_version(
        &self,
        mut_conn: &mut SqliteConnection,
        opt_event_id: Option<i32>,
    ) -> Result<(), EventSourcableError>;
}

#[derive(Debug, Clone)]
pub struct EventData {
    pub id: i32,
    pub opt_object_id: Option<i32>,
    pub opt_event_id: Option<i32>,
    pub opt_event_arg: Option<i32>,
    pub event_stack_level: i32,
    pub event_action: crate::autogen::schema::EventAction,
    pub created_on: String,
}

pub trait HasEventObject {
    /// The part of the event table that is duplicate of the base object
    type Object: Clone + Debug;
}

pub trait ReadEventTable: HasEventObject {
    /// Read all the events and decompose them into event data and object parts.
    /// if `opt_from_event_id` is not None, include only ids greater than it. This is done because everything <= it has
    /// been processed.
    fn read_event_table(
        &self,
        mut_conn: &mut SqliteConnection,
        opt_from_event_id: Option<i32>,
    ) -> Result<Vec<(EventData, Self::Object)>, EventSourcableError>;
}

pub trait InsertIntoHeadTable: HasEventObject {
    fn insert_into_head_table(
        &self,
        mut_conn: &mut SqliteConnection,
        objects: &[Self::Object],
    ) -> Result<(), EventSourcableError>;
}

pub trait UpdateHeadTableRow: HasEventObject {
    fn update_head_table_row(
        &self,
        mu_conn: &mut SqliteConnection,
        object_id: i32,
        object: Self::Object,
    ) -> Result<(), EventSourcableError>;
}

pub trait DeleteHeadTableRow {
    fn delete_head_table_row(
        &self,
        mut_conn: &mut SqliteConnection,
        object_id: i32,
    ) -> Result<(), EventSourcableError>;
}

pub trait ClearHeadTable {
    fn clear_head_table(&self, mut_conn: &mut SqliteConnection) -> Result<(), EventSourcableError>;
}

pub trait SourceHeadTable {
    /// This clears the table and sources the data from a base table
    fn source_head_table_row(
        &self,
        mut_conn: &mut SqliteConnection,
    ) -> Result<(), EventSourcableError>;
}

/// These capabilities allow us to manage a table head from its events
pub trait EventSourcable:
    ReadEventTableVersion
    + WriteEventTableVersion
    + ReadEventTable
    + InsertIntoHeadTable
    + UpdateHeadTableRow
    + DeleteHeadTableRow
    + ClearHeadTable
    + SourceHeadTable
{
}

#[derive(Error, Debug)]
pub enum SpawnEaBgWorkerThreadError {
    #[error("Could not find table name {0:?} in head access lock map")]
    TableNameNotInLockMap(String),

    #[error("Could not send message: {0:?}")]
    SendError(#[from] SendError<String>),

    #[error("Error while trying to receive message: {0:?}")]
    RecvError(#[from] RecvError),

    #[error("Got a mutex posion error: {0:?}")]
    PoisonError(String),

    #[error("Failed to connect to database: {0:?}")]
    EstablishConnectionError(#[from] EstablishConnectionError),

    #[error("Failed to perform event sourcable action: {0:?}")]
    EventSourcableError(#[from] EventSourcableError),
}

pub fn spawn_ea_bg_worker_thread<Src: EventSourcable + 'static + Send>(
    rx_ea_msg: Receiver<EaThreadMessage>,
    tx_work_done: Sender<String>,
    head_access_lock_map: HashMap<String, Arc<Mutex<()>>>,
    event_sourcable_map: HashMap<String, Src>,
) -> JoinHandle<Result<(), SpawnEaBgWorkerThreadError>> {
    thread::spawn(move || {
        let mut mut_conn = loader::establish_connection()?;

        loop {
            // Wait for work. A database write event means that new table events need to be accumulated.
            let message = rx_ea_msg.recv()?;

            match message {
                EaThreadMessage::DbWrite { table_name } => {
                    {
                        // Work has arrived!
                        info!("Work has arrived!");

                        // This is dropped at the end of this block, so we're safe until then to write!
                        let _lock = head_access_lock_map
                            .get(&table_name)
                            .ok_or(SpawnEaBgWorkerThreadError::TableNameNotInLockMap(
                                table_name.clone(),
                            ))?
                            .lock()
                            .map_err(|e| SpawnEaBgWorkerThreadError::PoisonError(e.to_string()))?;

                        let event_sourcable = event_sourcable_map.get(&table_name).ok_or(
                            SpawnEaBgWorkerThreadError::TableNameNotInLockMap(table_name.clone()),
                        )?;

                        let events = event_sourcable.read_event_table(&mut mut_conn, None)?;

                        let (_event_data, obj) = events[0].clone();

                        event_sourcable.update_head_table_row(&mut mut_conn, 1, obj)?;

                        // Generate the head table marked by table_name
                    }

                    // We are done with work! Let them know!
                    tx_work_done.send(table_name)?;
                }
                EaThreadMessage::DbCheckout {
                    table_name,
                    opt_event_id,
                } => {
                    info!(
                        "Checking out version {} for {table_name}",
                        match opt_event_id {
                            Some(event_id) => event_id.to_string(),
                            None => "latest".to_owned(),
                        }
                    );

                    // This is dropped at the end of this block, so we're safe until then to write!
                    let _lock = head_access_lock_map
                        .get(&table_name)
                        .ok_or(SpawnEaBgWorkerThreadError::TableNameNotInLockMap(
                            table_name.clone(),
                        ))?
                        .lock()
                        .map_err(|e| SpawnEaBgWorkerThreadError::PoisonError(e.to_string()))?;
                }

                EaThreadMessage::Quit => break,
            }
        }

        Ok(())
    })
}

mod actions {}
