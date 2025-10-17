#[derive(diesel_derive_enum::DbEnum, Debug, strum::VariantArray, Clone)]
pub enum EventAction {
    Insert,
    Update,
    Delete,
    Open,
    Close,
    Reopen,
}

#[derive(diesel_derive_enum::DbEnum, Debug, strum::VariantArray, Clone)]
pub enum ObjState {
    Insert,
    Update,
    Delete,
}

impl From<ObjState> for EventAction {
    fn from(value: ObjState) -> Self {
        match value {
            ObjState::Insert => Self::Insert,
            ObjState::Update => Self::Update,
            ObjState::Delete => Self::Delete,
        }
    }
}

// @generated automatically by Diesel CLI.

diesel::table! {
    coin_store_diffs (id) {
        id -> Integer,
        obj_id -> Integer,
        person -> Text,
        coins -> Integer,
    }
}

diesel::table! {
    coin_store_events (id) {
        id -> Integer,
        opt_diff_id -> Nullable<Integer>,
        ev_action -> crate::autogen::schema::EventActionMapping,
        span -> Integer,
        frame -> Integer,
        created_on_ts -> Float,
        ev_desc -> Text,
    }
}

diesel::table! {
    coin_store_events_grouped (id) {
        id -> Integer,
        grp_id -> Integer,
        grp_span -> Integer,
        grp_frame -> Integer,
        grp_created_on_ts -> Float,
        dup -> Integer,
        ev_id -> Integer,
        obj_id -> Integer,
        ev_action -> crate::autogen::schema::EventActionMapping,
        span -> Integer,
        frame -> Integer,
        created_on_ts -> Float,
        person -> Text,
        coins -> Integer,
        ev_desc -> Text,
    }
}

diesel::table! {
    coin_store_events_grouped_partial (id) {
        id -> Integer,
        grp_id -> Integer,
        grp_span -> Integer,
        grp_frame -> Integer,
        grp_created_on_ts -> Float,
        dup -> Integer,
        ev_id -> Integer,
        obj_id -> Integer,
        ev_action -> crate::autogen::schema::EventActionMapping,
        span -> Integer,
        frame -> Integer,
        created_on_ts -> Float,
        person -> Text,
        coins -> Integer,
        ev_desc -> Text,
    }
}

diesel::table! {
    coin_store_hist (id) {
        id -> Integer,
        grp_id -> Integer,
        grp_span -> Integer,
        grp_frame -> Integer,
        obj_id -> Integer,
        obj_state -> crate::autogen::schema::ObjStateMapping,
        person -> Text,
        coins -> Integer,
    }
}

diesel::table! {
    coin_store_hist_partial (id) {
        id -> Integer,
        grp_id -> Integer,
        grp_span -> Integer,
        grp_frame -> Integer,
        obj_id -> Integer,
        obj_state -> crate::autogen::schema::ObjStateMapping,
        person -> Text,
        coins -> Integer,
    }
}

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

diesel::joinable!(coin_store_events -> coin_store_diffs (opt_diff_id));

diesel::allow_tables_to_appear_in_same_query!(
    coin_store_diffs,
    coin_store_events,
    coin_store_events_grouped,
    coin_store_events_grouped_partial,
    coin_store_hist,
    coin_store_hist_partial,
    credit_store,
    credit_store_events,
    credit_store_head,
    credit_store_version,
);
