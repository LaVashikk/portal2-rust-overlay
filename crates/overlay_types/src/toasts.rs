use std::sync::LazyLock;

use egui::Color32;

use crate::events::OverlayEvent;

#[derive(Clone)]
pub struct Toast {
    pub text: String,
    pub color: Color32,
    pub time_left: f32,
}

#[derive(Default, Clone)]
pub struct Toaster {
    pub(crate) toasts: Vec<Toast>,
}

// TODO
// static TOASTS: LazyLock<Toaster> = LazyLock::new(Toaster::default);

// pub fn success(text: impl Into<String>) {
//     TOASTS.toasts.push(Toast { text: text.into(), color: Color32::GREEN, time_left: 3.0 });
// }

// pub fn error(text: impl Into<String>) {
//     TOASTS.toasts.push(Toast { text: text.into(), color: Color32::RED, time_left: 4.0 });
// }

// pub fn info(text: impl Into<String>) {
//     TOASTS.toasts.push(Toast { text: text.into(), color: Color32::LIGHT_BLUE, time_left: 3.0 });
// }
