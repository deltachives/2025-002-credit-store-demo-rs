// Coin Store

use diesel::expression::AsExpression;
use diesel_derive_newtype::*;
use thiserror::Error;

#[derive(Debug, Clone, Hash, PartialEq, Eq, DieselNewType)]
pub struct Person(String);

#[derive(Error, Debug, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum PersonNewError {
    #[error("Person name cannot be admin")]
    AdminNotAllowed,
}

impl Person {
    pub fn new(s: &str) -> Result<Self, PersonNewError> {
        if s.to_lowercase() == "admin" {
            Err(PersonNewError::AdminNotAllowed)
        } else {
            Ok(Person(s.to_owned()))
        }
    }
}

pub mod coin_store {
    use diesel::prelude::*;

    crate::macros::diesel_hist_models::create_diesel_hist_structs_read! {
        diff_table: coin_store_diffs,
        events_table: coin_store_events,
        events_grouped_table: coin_store_events_grouped,
        events_grouped_partial_table: coin_store_events_grouped_partial,
        hist_table: coin_store_hist,
        hist_partial_table: coin_store_hist_partial,

        fields_read: {
            person: super::Person,
            coins: i32,
        }
    }

    crate::macros::diesel_hist_models::create_diesel_hist_structs_write_one_lifetime! {
        diff_table: coin_store_diffs,
        events_table: coin_store_events,
        events_grouped_table: coin_store_events_grouped,
        events_grouped_partial_table: coin_store_events_grouped_partial,
        hist_table: coin_store_hist,
        hist_partial_table: coin_store_hist_partial,

        fields_write: {
            coins: i32,
        },

        fields_write_ref: {
            person: &'a super::Person,
        },
    }
}
