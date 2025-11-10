use std::fs::File;
use log::LevelFilter;
use simplelog::{CombinedLogger, Config, TermLogger, WriteLogger, TerminalMode, ColorChoice};

/// Available values: Off, Error, Warn, Info, Debug, Trace
const LOG_LEVEL: LevelFilter = LevelFilter::Info;

/// Path to the log file.
const LOG_FILE_PATH: &str = "d3d9_proxy_mod.log";

/// Initializes the logging system.
pub fn init() {
    let log_file = File::create(LOG_FILE_PATH);
    let result = match log_file {
        Ok(file) => {
            CombinedLogger::init(vec![
                // Logger for the terminal (stderr) with colored output.
                TermLogger::new(
                    LOG_LEVEL,
                    Config::default(),
                    TerminalMode::Mixed,
                    ColorChoice::Auto,
                ),
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
