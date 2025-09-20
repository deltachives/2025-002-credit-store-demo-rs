use diesel::prelude::*;

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
