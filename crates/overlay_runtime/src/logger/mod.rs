use std::{fs::File, sync::atomic::{AtomicBool, Ordering}};
use log::LevelFilter;
use simplelog::{CombinedLogger, Config, TermLogger, WriteLogger, TerminalMode, ColorChoice};

/// Available values: Off, Error, Warn, Info, Debug, Trace
const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

/// Path to the log file.
const LOG_FILE_PATH: &str = "survey_playtest_addon.log";

mod to_toasts;
mod game_console;
pub use game_console::PRINT_LOGS_IN_GAME_CONSOLE;

/// Initializes the logging system.
pub fn init() {
    let log_file = File::create(LOG_FILE_PATH);

    let result = match log_file {
        Ok(file) => {
            CombinedLogger::init(vec![
                // Logger for the terminal (stderr) with colored output
                TermLogger::new(
                    LOG_LEVEL,
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),

                // Logger to game-console
                game_console::PortalConsoleLogger::new(),

                // Repeat log info in egui-toasts
                // Use `log::warn!(target: "toast", "log")` for this
                to_toasts::ToToastsLogger::new(),

                // Logger to file
                WriteLogger::new(
                    LOG_LEVEL,
                    Config::default(),
                    file,
                ),
            ])
        }
        Err(_) => {
            TermLogger::init(
                LOG_LEVEL,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            )
        }
    };

    if result.is_err() {
        log::error!("Failed to initialize logger!");
    }
}
