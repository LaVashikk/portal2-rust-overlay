use std::sync::atomic::{AtomicBool, Ordering};
use log::{Level, LevelFilter};
use simplelog::Config ;

const GAME_CONSOLE_PREFIX: &str = "PLUGIN";
pub static PRINT_LOGS_IN_GAME_CONSOLE: AtomicBool = AtomicBool::new(true);

pub struct PortalConsoleLogger;

impl PortalConsoleLogger {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl log::Log for PortalConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= super::LOG_LEVEL && PRINT_LOGS_IN_GAME_CONSOLE.load(Ordering::Relaxed)
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let message = record.args().to_string();
        match record.level() {
            Level::Error => {
                portal2_sdk::con_color_print!(
                    portal2_sdk::Color::BRIGHT_RED,
                    "[{GAME_CONSOLE_PREFIX}-ERROR] {}\n", message
                );
            }
            Level::Warn => {
                portal2_sdk::con_color_print!(
                    portal2_sdk::Color::ORANGE,
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

    fn flush(&self) {}
}

impl simplelog::SharedLogger for PortalConsoleLogger {
    fn level(&self) -> LevelFilter {
        super::LOG_LEVEL
    }

    fn config(&self) -> Option<&Config> {
        None
    }

    fn as_log(self: Box<Self>) -> Box<dyn log::Log> {
        Box::new(*self)
    }
}
