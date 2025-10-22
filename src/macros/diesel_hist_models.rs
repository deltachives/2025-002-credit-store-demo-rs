/// Should not be constructed manually. This signals the invariant of an existent non-duplicate span frame in db
/// See `create_span_frame` functions for creating a spanframe and `get_created_span_frames` for getting them.
/// There are also `close_span_frame` and `reopen_span_frame` options.
#[derive(Debug, Clone)]
pub struct SpanFrame {
    pub span: i32,
    pub frame: i32,
}

#[derive(thiserror::Error, Debug)]
pub enum CreateSpanFrameError {
    #[error("Diesel Error: {0:?}")]
    DieselError(#[from] diesel::result::Error),

    #[error("Duplicate span frame at span={span} frame={frame}")]
    DuplicateSpanFrame { span: i32, frame: i32 },
}

macro_rules! create_diesel_hist_structs_read {
    {
        diff_table: $diff_table:ident,
        events_table: $events_table:ident,
        events_grouped_table: $events_grouped_table:ident,
        events_grouped_partial_table: $events_grouped_partial_table:ident,
        hist_table: $hist_table:ident,
        hist_partial_table: $hist_partial_table:ident,

        fields_read: {$($field_read:ident: $typ_read:ty),+ $(,)?}$(,)?
    } => {
        #[derive(Debug)]
        #[allow(dead_code)]
        pub struct Common {
            $(
                pub $field_read: $typ_read,
            )*
        }

        #[allow(dead_code)]
        pub trait GetCommon {
            fn get_common(&self) -> Common;
        }

        #[derive(Debug, Queryable, Selectable)]
        #[diesel(table_name = crate::autogen::schema::$diff_table)]
        #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
        #[allow(dead_code)]
        pub struct Diff {
            pub id: i32,
            pub obj_id: i32,
            $(
                pub $field_read: $typ_read,
            )*
        }

        impl GetCommon for Diff {
            fn get_common(&self) -> Common {
                Common {
                    $(
                        $field_read: self.$field_read.clone(),
                    )*
                }
            }
        }

        #[derive(Debug, Queryable, Selectable)]
        #[diesel(table_name = crate::autogen::schema::$events_table)]
        #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
        #[allow(dead_code)]
        pub struct Event {
            pub id: i32,
            pub opt_diff_id: Option<i32>,
            pub ev_action: crate::autogen::schema::EventAction,
            pub span: i32,
            pub frame: i32,
            pub created_on_ts: f32,
            pub ev_desc: String,
        }

        #[derive(Debug, Queryable, Selectable)]
        #[diesel(table_name = crate::autogen::schema::$events_grouped_table)]
        #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
        #[allow(dead_code)]
        pub struct EventGrouped {
            pub id: i32,
            pub grp_id: i32,
            pub grp_span: i32,
            pub grp_frame: i32,
            pub grp_created_on_ts: f32,
            pub dup: i32,
            pub ev_id: i32,
            pub obj_id: i32,
            pub ev_action: crate::autogen::schema::EventAction,
            pub span: i32,
            pub frame: i32,
            pub created_on_ts: f32,
            $(
                pub $field_read: $typ_read,
            )*
            pub ev_desc: String,
        }

        impl GetCommon for EventGrouped {
            fn get_common(&self) -> Common {
                Common {
                    $(
                        $field_read: self.$field_read.clone(),
                    )*
                }
            }
        }


        #[derive(Debug, Queryable, Selectable)]
        #[diesel(table_name = crate::autogen::schema::$events_grouped_partial_table)]
        #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
        #[allow(dead_code)]
        pub struct EventGroupedPartial {
            pub id: i32,
            pub grp_id: i32,
            pub grp_span: i32,
            pub grp_frame: i32,
            pub grp_created_on_ts: f32,
            pub dup: i32,
            pub ev_id: i32,
            pub obj_id: i32,
            pub ev_action: crate::autogen::schema::EventAction,
            pub span: i32,
            pub frame: i32,
            pub created_on_ts: f32,
            $(
                pub $field_read: $typ_read,
            )*
            pub ev_desc: String,
        }

        impl GetCommon for EventGroupedPartial {
            fn get_common(&self) -> Common {
                Common {
                    $(
                        $field_read: self.$field_read.clone(),
                    )*
                }
            }
        }

        #[derive(Debug, Queryable, Selectable)]
        #[diesel(table_name = crate::autogen::schema::$hist_table)]
        #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
        #[allow(dead_code)]
        pub struct Hist {
            pub id: i32,
            pub grp_id: i32,
            pub grp_span: i32,
            pub grp_frame: i32,
            pub obj_id: i32,
            pub obj_state: crate::autogen::schema::ObjState,
            $(
                pub $field_read: $typ_read,
            )*
        }

        impl GetCommon for Hist {
            fn get_common(&self) -> Common {
                Common {
                    $(
                        $field_read: self.$field_read.clone(),
                    )*
                }
            }
        }

        #[derive(Debug, Queryable, Selectable)]
        #[diesel(table_name = crate::autogen::schema::$hist_partial_table)]
        #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
        #[allow(dead_code)]
        pub struct HistPartial {
            pub id: i32,
            pub grp_id: i32,
            pub grp_span: i32,
            pub grp_frame: i32,
            pub obj_id: i32,
            pub obj_state: crate::autogen::schema::ObjState,
            $(
                pub $field_read: $typ_read,
            )*
        }

        impl GetCommon for HistPartial {
            fn get_common(&self) -> Common {
                Common {
                    $(
                        $field_read: self.$field_read.clone(),
                    )*
                }
            }
        }
    }
}

/// Supports lifetime 'a
macro_rules! create_diesel_hist_structs_write_one_lifetime {
    {
        diff_table: $diff_table:ident,
        events_table: $events_table:ident,
        events_grouped_table: $events_grouped_table:ident,
        events_grouped_partial_table: $events_grouped_partial_table:ident,
        hist_table: $hist_table:ident,
        hist_partial_table: $hist_partial_table:ident,

        fields_write: {$($field_write:ident: $typ_write:ty),+ $(,)?}$(,)?
        fields_write_ref: {$($field_write_ref:ident: $typ_write_ref:ty),+ $(,)?}$(,)?
    } => {

        #[derive(Debug)]
        #[allow(dead_code)]
        pub struct NewCommon<'a> {
            $(
                pub $field_write: $typ_write,
            )*
            $(
                pub $field_write_ref: $typ_write_ref,
            )*
        }

        #[derive(Debug, Insertable, AsChangeset)]
        #[diesel(table_name = crate::autogen::schema::$diff_table)]
        #[allow(dead_code)]
        pub struct NewDiff<'a> {
            pub obj_id: i32,
            $(
                pub $field_write: $typ_write,
            )*
            $(
                pub $field_write_ref: $typ_write_ref,
            )*
        }

        impl<'a> NewDiff<'a> {
            pub fn new_with_common(obj_id: i32, new_common: NewCommon<'a>) -> NewDiff<'a> {
                NewDiff {
                    obj_id,
                    $(
                        $field_write: new_common.$field_write,
                    )*
                    $(
                        $field_write_ref: new_common.$field_write_ref,
                    )*
                }
            }
        }

        #[derive(Debug, Insertable, AsChangeset)]
        #[diesel(table_name = crate::autogen::schema::$events_table)]
        #[allow(dead_code)]
        pub struct NewEvent<'a> {
            pub opt_diff_id: Option<i32>,
            pub ev_action: crate::autogen::schema::EventAction,
            pub span: i32,
            pub frame: i32,
            pub created_on_ts: f32,
            pub ev_desc: &'a str,
        }

        #[derive(Debug, Insertable, AsChangeset)]
        #[diesel(table_name = crate::autogen::schema::$events_grouped_partial_table)]
        #[allow(dead_code)]
        pub struct NewEventGroupedPartial<'a> {
            pub grp_id: i32,
            pub grp_span: i32,
            pub grp_frame: i32,
            pub grp_created_on_ts: f32,
            pub dup: i32,
            pub ev_id: i32,
            pub obj_id: i32,
            pub ev_action: crate::autogen::schema::EventAction,
            pub span: i32,
            pub frame: i32,
            pub created_on_ts: f32,
            $(
                pub $field_write: $typ_write,
            )*
            $(
                pub $field_write_ref: $typ_write_ref,
            )*
            pub ev_desc: &'a str,
        }

        pub fn get_created_span_frames(conn: &mut SqliteConnection) -> Result<Vec<crate::macros::diesel_hist_models::SpanFrame>, diesel::result::Error> {
            use crate::autogen::schema::$events_table::dsl::*;

            let frame_events: Vec<Event> = $events_table
                .filter(ev_action.eq(crate::autogen::schema::EventAction::Open))
                .select(Event::as_select())
                .get_results(conn)?;

            let span_frames = frame_events
                .into_iter()
                .map(|event| crate::macros::diesel_hist_models::SpanFrame { span: event.span, frame: event.frame })
                .collect::<Vec<_>>();

            Ok(span_frames)
        }

        #[allow(dead_code)]
        pub fn create_span_frame(
            conn: &mut SqliteConnection,
            span: i32,
            frame: i32,
            ev_desc: &str
        ) -> Result<crate::macros::diesel_hist_models::SpanFrame, crate::macros::diesel_hist_models::CreateSpanFrameError> {
            use chrono::prelude::*;

            let span_frames = get_created_span_frames(conn)?;

            let duplicate = span_frames
                .into_iter()
                .any(|span_frame| span_frame.span == span && span_frame.frame == frame);

            if duplicate {
                return Err(crate::macros::diesel_hist_models::CreateSpanFrameError::DuplicateSpanFrame { span, frame });
            }

            let new_event = NewEvent {
                opt_diff_id: None,
                ev_action: crate::autogen::schema::EventAction::Open,
                span,
                frame,
                created_on_ts: Utc::now().timestamp_millis() as f32,
                ev_desc,
            };

            let out = diesel::insert_into(crate::autogen::schema::$events_table::dsl::$events_table)
                .values(&new_event)
                .returning(Event::as_returning())
                .get_result(conn)?;

            Ok(crate::macros::diesel_hist_models::SpanFrame { span: out.span, frame: out.frame })
        }

        #[allow(dead_code)]
        pub fn close_span_frame(conn: &mut SqliteConnection, span_frame: crate::macros::diesel_hist_models::SpanFrame, ev_desc: &str) -> Result<(), diesel::result::Error> {
            use chrono::prelude::*;

            let new_event = NewEvent {
                opt_diff_id: None,
                ev_action: crate::autogen::schema::EventAction::Close,
                span: span_frame.span,
                frame: span_frame.frame,
                created_on_ts: Utc::now().timestamp_millis() as f32,
                ev_desc,
            };

            diesel::insert_into(crate::autogen::schema::$events_table::dsl::$events_table)
                .values(&new_event)
                .returning(Event::as_returning())
                .execute(conn)?;

            Ok(())
        }

        #[allow(dead_code)]
        pub fn reopen_span_frame(conn: &mut SqliteConnection, span_frame: crate::macros::diesel_hist_models::SpanFrame, ev_desc: &str) -> Result<(), diesel::result::Error> {
            use chrono::prelude::*;

            let new_event = NewEvent {
                opt_diff_id: None,
                ev_action: crate::autogen::schema::EventAction::Reopen,
                span: span_frame.span,
                frame: span_frame.frame,
                created_on_ts: Utc::now().timestamp_millis() as f32,
                ev_desc,
            };

            diesel::insert_into(crate::autogen::schema::$events_table::dsl::$events_table)
                .values(&new_event)
                .returning(Event::as_returning())
                .execute(conn)?;

            Ok(())
        }

        fn insert_diff<'a>(conn: &mut SqliteConnection, obj_id: i32, new_common: NewCommon<'a>) -> Result<Diff, diesel::result::Error> {
            let new_diff = NewDiff::new_with_common(obj_id, new_common);

            let out = diesel::insert_into(crate::autogen::schema::$diff_table::dsl::$diff_table)
                .values(&new_diff)
                .returning(Diff::as_returning())
                .get_result(conn)?;

            Ok(out)
        }

        #[allow(dead_code)]
        pub fn insert_event_for_obj<'a>(
            conn: &mut SqliteConnection,
            obj_id: i32,
            span_frame: &crate::macros::diesel_hist_models::SpanFrame,
            obj_state: crate::autogen::schema::ObjState,
            ev_desc: &'a str,
            new_common: NewCommon<'a>
        ) -> Result<Event, diesel::result::Error> {
            use chrono::prelude::*;

            // Check that the frame exists


            let diff = insert_diff(conn, obj_id, new_common)?;

            let new_event = NewEvent {
                opt_diff_id: Some(diff.id),
                ev_action: obj_state.into(),
                span: span_frame.span,
                frame: span_frame.frame,
                created_on_ts: Utc::now().timestamp_millis() as f32,
                ev_desc,
            };

            let out = diesel::insert_into(crate::autogen::schema::$events_table::dsl::$events_table)
                .values(&new_event)
                .returning(Event::as_returning())
                .get_result(conn)?;

            Ok(out)
        }

        #[allow(dead_code)]
        pub fn set_events_grouped_partial(
            conn: &mut SqliteConnection,
            events_grouped: &[EventGrouped]
        ) -> Result<(), diesel::result::Error> {

            diesel::delete(crate::autogen::schema::$events_grouped_partial_table::dsl::$events_grouped_partial_table)
                .execute(conn)?;

            let new_events_grouped_partial = events_grouped
                .into_iter()
                .map(|e| {
                    let new_common = NewCommon {
                        $(
                            $field_write: e.$field_write,
                        )*
                        $(
                            $field_write_ref: &e.$field_write_ref,
                        )*
                    };

                    NewEventGroupedPartial {
                        grp_id: e.grp_id,
                        grp_span: e.grp_span,
                        grp_frame: e.grp_frame,
                        grp_created_on_ts: e.grp_created_on_ts,
                        dup: e.dup,
                        ev_id: e.ev_id,
                        obj_id: e.obj_id,
                        ev_action: e.ev_action.clone(),
                        span: e.span,
                        frame: e.frame,
                        created_on_ts: e.created_on_ts,
                        $(
                            $field_write: new_common.$field_write,
                        )*
                        $(
                            $field_write_ref: new_common.$field_write_ref,
                        )*
                        ev_desc: &e.ev_desc
                    }
                })
                .collect::<Vec<_>>();

            diesel::insert_into(crate::autogen::schema::$events_grouped_partial_table::dsl::$events_grouped_partial_table)
                .values(new_events_grouped_partial)
                .execute(conn)?;

            Ok(())
        }
    }
}

pub(crate) use create_diesel_hist_structs_read;
pub(crate) use create_diesel_hist_structs_write_one_lifetime;
