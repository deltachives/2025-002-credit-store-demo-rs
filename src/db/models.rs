use diesel::prelude::*;

// Credit Store

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::autogen::schema::credit_store)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CreditStoreRec {
    pub id: i32,
    pub person: String,
    pub credits: i32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::autogen::schema::credit_store_head)]
pub struct NewCreditStoreRec<'a> {
    pub person: &'a str,
    pub credits: i32,
}

#[derive(Debug, Clone)]
pub struct CreditStoreObject {
    pub person: String,
    pub credits: i32,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::autogen::schema::credit_store_head)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CreditStoreHeadRec {
    pub id: i32,
    pub person: String,
    pub credits: i32,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::autogen::schema::credit_store_version)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CreditStoreVersion {
    pub id: i32,
    pub opt_event_id: Option<i32>,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = crate::autogen::schema::credit_store_version)]
pub struct NewCreditStoreVersion {
    pub opt_event_id: Option<i32>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::autogen::schema::credit_store_events)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CreditStoreEvent {
    pub id: i32,
    pub person: String,
    pub credits: i32,
    pub opt_object_id: Option<i32>,
    pub opt_event_id: Option<i32>,
    pub opt_event_arg: Option<i32>,
    pub event_stack_level: i32,
    pub event_action: crate::autogen::schema::EventAction,
    pub created_on: String,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = crate::autogen::schema::credit_store_events)]
pub struct NewCreditStoreEvent<'a> {
    pub person: String,
    pub credits: i32,
    pub opt_object_id: Option<i32>,
    pub opt_event_id: Option<i32>,
    pub opt_event_arg: Option<i32>,
    pub event_stack_level: i32,
    pub event_action: crate::autogen::schema::EventAction,
    pub created_on: &'a str,
}

// Coin Store

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::autogen::schema::coin_store_diffs)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CoinStoreDiff {
    pub id: i32,
    pub obj_id: i32,
    pub person: String,
    pub coins: i32,
}

#[derive(Debug, Insertable, AsChangeset)]
#[diesel(table_name = crate::autogen::schema::coin_store_diffs)]
pub struct NewCoinStoreDiff<'a> {
    pub obj_id: i32,
    pub person: &'a str,
    pub coins: i32,
}

mod coin_store {
    use diesel::prelude::*;

    crate::macros::diesel_hist_models::create_diesel_hist_structs_read! {
        diff_table: coin_store_diffs,
        events_table: coin_store_events,
        events_grouped_table: coin_store_events_grouped,
        events_grouped_partial_table: coin_store_events_grouped_partial,
        hist_table: coin_store_hist,
        hist_partial_table: coin_store_hist_partial,

        fields_read: {
            person: String,
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
            person: &'a str,
        },
    }
}
