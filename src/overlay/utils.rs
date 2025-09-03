use egui::Context;
/// Trait that every window must implement.
#[allow(dead_code)]
pub trait Window: std::fmt::Debug { // todo debug
    /// The name of the window, used for the title.
    fn name(&self) -> &'static str;

    /// Shows or hides the window.
    fn toggle(&mut self);
    /// todo
    // fn enable(&mut self);

    /// Returns whether the window is open.
    fn is_open(&self) -> bool;

    /// The drawing logic of the window.
    fn draw(&mut self, ctx: &Context, shared_state: &mut super::SharedState);
}

// SAFETY: Used within `UiManager`, which is protected by a Mutex and lazily initialized.
//          This ensures that access to the underlying `InputContextT` is synchronized.
//
// Alternative considered: `thread_local!`, but that would be fatal if called from a rendering thread.
//          This approach is preferred because it provides synchronization, even if it introduces some overhead.
pub struct SendableContext(pub *mut crate::engine::input_system::InputContextT);
unsafe impl Send for SendableContext {}
unsafe impl Sync for SendableContext {}
