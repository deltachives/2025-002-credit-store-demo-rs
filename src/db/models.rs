// Coin Store

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
