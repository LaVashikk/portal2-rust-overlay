use std::sync::atomic::{AtomicBool, Ordering};
use log::{Level, LevelFilter};
use simplelog::Config ;

pub struct ToToastsLogger;

impl ToToastsLogger {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl log::Log for ToToastsLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= super::LOG_LEVEL
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) || record.target() != "toast" {
            return;
        }

        let message = record.args().to_string();
        match record.level() {
            Level::Error => {
                overlay_types::toasts::error(message, 5000);
            }
            Level::Warn => {
                overlay_types::toasts::warning(message, 3000);
            }
            Level::Info => {
                overlay_types::toasts::info(message, 1500);
            }
            _ => {}
        }
    }

    fn flush(&self) {}
}

impl simplelog::SharedLogger for ToToastsLogger {
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
