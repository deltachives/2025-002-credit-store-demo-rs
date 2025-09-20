//! Handles logging related setup and use

use log::LevelFilter;

pub fn init_logging_with_level(level: LevelFilter) {
    env_logger::builder()
        .filter_level(level)
        .filter_module("rustyline", LevelFilter::Warn)
        .try_init()
        .map_err(|e| e.to_string())
        .expect("Failed to initialize logger");
}
