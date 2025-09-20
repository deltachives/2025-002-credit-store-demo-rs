use crate::db::models::*;
use diesel::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbActionError {
    #[error("Table action error: {0:?}")]
    DieselError(#[from] diesel::result::Error),
}

pub fn insert_credit_store_event(
    conn: &mut SqliteConnection,
    object: NewCreditStoreEvent,
) -> Result<CreditStoreEvent, DbActionError> {
    use crate::autogen::schema::credit_store_events::dsl;

    let out = diesel::insert_into(dsl::credit_store_events)
        .values(&object)
        .returning(CreditStoreEvent::as_returning())
        .get_result(conn)?;

    Ok(out)
}
