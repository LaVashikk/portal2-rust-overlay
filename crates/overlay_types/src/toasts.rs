use std::{collections::VecDeque, sync::{LazyLock, Mutex}, time::Duration};
use egui::{Color32, RichText, Vec2};
use egui_notify::{Toast, Toasts};
use crate::events::OverlayEvent;

const TOAST_FONT_SIZE: f32 = 18.0;
pub static TOASTS: LazyLock<Mutex<Toasts>> = LazyLock::new(|| Mutex::new(
    Toasts::default().with_margin(Vec2::new(10., 35.))
));

fn add_toast(text: impl Into<String>, duration_ms: u64, constructor: fn(RichText) -> Toast) {
    if let Ok(mut toasts) = TOASTS.lock() {
        let caption = RichText::new(text.into())
            .size(TOAST_FONT_SIZE)
            .color(Color32::WHITE);

        let mut toast = constructor(caption);
        toast.duration(Some(Duration::from_millis(duration_ms)));
        toasts.add(toast);
    }
}

pub fn basic(text: impl Into<String>, duration_ms: u64)   { add_toast(text, duration_ms, Toast::basic); }
pub fn info(text: impl Into<String>, duration_ms: u64)    { add_toast(text, duration_ms, Toast::info); }
pub fn success(text: impl Into<String>, duration_ms: u64) { add_toast(text, duration_ms, Toast::success); }
pub fn warning(text: impl Into<String>, duration_ms: u64) { add_toast(text, duration_ms, Toast::warning); }
pub fn error(text: impl Into<String>, duration_ms: u64)   { add_toast(text, duration_ms, Toast::error); }
