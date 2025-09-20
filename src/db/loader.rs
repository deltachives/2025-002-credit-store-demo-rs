use diesel::prelude::*;
use dotenvy::dotenv;
use std::env::{self, VarError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EstablishConnectionError {
    #[error("Failed to setup dotenvy: {0:?}")]
    DotEnvyError(#[from] dotenvy::Error),

    #[error("Failed to retrieve environmental variable: {0:?}")]
    VarError(#[from] VarError),

    #[error("Failed to connect to the database: {0:?}")]
    ConnectionError(#[from] ConnectionError),
}

pub fn establish_connection() -> Result<SqliteConnection, EstablishConnectionError> {
    dotenv()?;

    let database_url = env::var("DATABASE_URL")?;

    Ok(SqliteConnection::establish(&database_url)?)
}
