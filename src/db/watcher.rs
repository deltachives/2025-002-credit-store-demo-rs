//! This module handles subscribing to database events. It assumes that control is only by the
//! current application, and so it allows it to publish events for subscribers.

pub struct DbEventTopic {
    _table_name: String,
}

pub fn publish_db_event(_topic: DbEventTopic) {}
