use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbActionError {
    #[error("Table action error: {0:?}")]
    DieselError(#[from] diesel::result::Error),
}
