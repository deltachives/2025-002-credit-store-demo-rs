#[derive(diesel_derive_enum::DbEnum, Debug, strum::VariantArray, Clone)]
pub enum EventAction {
    Insert,
    Update,
    Delete,
    Pop,
    Reset,
    Init,
    Undo,
    Redo,
    Seek,
}

// @generated automatically by Diesel CLI.

diesel::table! {
    credit_store (id) {
        id -> Integer,
        person -> Text,
        credits -> Integer,
    }
}

diesel::table! {
    credit_store_events (id) {
        id -> Integer,
        person -> Text,
        credits -> Integer,
        event_action -> crate::autogen::schema::EventActionMapping,
        opt_object_id -> Nullable<Integer>,
        opt_event_id -> Nullable<Integer>,
        opt_event_arg -> Nullable<Integer>,
        event_stack_level -> Integer,
        created_on -> Text,
    }
}

diesel::table! {
    credit_store_head (id) {
        id -> Integer,
        person -> Text,
        credits -> Integer,
    }
}

diesel::table! {
    credit_store_version (id) {
        id -> Integer,
        opt_event_id -> Nullable<Integer>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    credit_store,
    credit_store_events,
    credit_store_head,
    credit_store_version,
);
