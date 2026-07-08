use std::{fs::File, sync::atomic::{AtomicBool, Ordering}};
use log::{Level, LevelFilter};
use simplelog::{CombinedLogger, Config, TermLogger, WriteLogger, TerminalMode, ColorChoice};

/// Available values: Off, Error, Warn, Info, Debug, Trace
const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

/// Path to the log file.
const LOG_FILE_PATH: &str = "d3d9_proxy_mod.log";

struct PortalConsoleLogger;
const GAME_CONSOLE_PREFIX: &str = "PLUGIN";
// todo: create a cvar handler for this
pub static PRINT_LOGS_IN_GAME_CONSOLE: AtomicBool = AtomicBool::new(true);

impl log::Log for PortalConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= LOG_LEVEL && PRINT_LOGS_IN_GAME_CONSOLE.load(Ordering::Relaxed)
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let Some(message) = record.args().as_str() else {
                return;
            };

            match record.level() {
                Level::Error => {
                    portal2_sdk::con_color_print!(
                        portal2_sdk::Color::rgb(201, 74, 74),
                        "[{GAME_CONSOLE_PREFIX}-ERROR] {}\n", message
                    );
                }
                Level::Warn => {
                    portal2_sdk::con_color_print!(
                        portal2_sdk::Color::rgb(237, 162, 55),
                        "[{GAME_CONSOLE_PREFIX}-WARN] {}\n", message
                    );
                }
                Level::Info => {
                    portal2_sdk::con_print!(
                        "[{GAME_CONSOLE_PREFIX}] {}\n", message
                    );
                }
                _ => {}
            }
        }
    }

    fn flush(&self) {}
}

impl simplelog::SharedLogger for PortalConsoleLogger {
    fn level(&self) -> LevelFilter {
        LOG_LEVEL
    }

    fn config(&self) -> Option<&Config> {
        None
    }

    fn as_log(self: Box<Self>) -> Box<dyn log::Log> {
        Box::new(*self)
    }
}

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
                Box::new(PortalConsoleLogger),

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
