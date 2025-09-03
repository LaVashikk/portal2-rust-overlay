
#[repr(C)]
pub struct InputContextT { _private: [u8; 0] }

// Opaque type for the `this` pointer.
#[repr(C)] pub(crate) struct RawIInputStackSystem { _private: [u8; 0] }

// --- Function Pointer Type Definitions ---
type FnPushInputContext = unsafe extern "thiscall" fn(this: *mut RawIInputStackSystem) -> *mut InputContextT;
type FnEnableInputContext = unsafe extern "thiscall" fn(this: *mut RawIInputStackSystem, context: *mut InputContextT, enable: bool);
type FnSetCursorVisible = unsafe extern "thiscall" fn(this: *mut RawIInputStackSystem, context: *mut InputContextT, enable: bool);
type FnSetMouseCapture = unsafe extern "thiscall" fn(this: *mut RawIInputStackSystem, context: *mut InputContextT, enable: bool);
type FnSetCursorPosition = unsafe extern "thiscall" fn(this: *mut RawIInputStackSystem, context: *mut InputContextT, x: i32, y: i32);
type FnIsTopmostEnabledContext = unsafe extern "thiscall" fn(this: *mut RawIInputStackSystem, context: *mut InputContextT) -> bool;

/// Represents an instance of the IInputStackSystem interface.
pub struct IInputStackSystem {
    pub(crate) this: *mut RawIInputStackSystem,

    pub(crate) push_input_context: FnPushInputContext,
    pub(crate) enable_input_context: FnEnableInputContext,
    pub(crate) set_cursor_visible: FnSetCursorVisible,
    pub(crate) set_mouse_capture: FnSetMouseCapture,
    pub(crate) set_cursor_position: FnSetCursorPosition,
    pub(crate) is_topmost_enabled_context: FnIsTopmostEnabledContext,
}

impl IInputStackSystem {
    pub fn push_input_context(&self) -> *mut InputContextT {
        unsafe { (self.push_input_context)(self.this) }
    }
    pub fn enable_input_context(&self, context: *mut InputContextT, enable: bool) {
        unsafe { (self.enable_input_context)(self.this, context, enable) }
    }
    pub fn set_cursor_visible(&self, context: *mut InputContextT, enable: bool) {
        unsafe { (self.set_cursor_visible)(self.this, context, enable) }
    }
    pub fn set_mouse_capture(&self, context: *mut InputContextT, enable: bool) {
        unsafe { (self.set_mouse_capture)(self.this, context, enable) }
    }
    pub fn set_cursor_position(&self, context: *mut InputContextT, x: i32, y: i32) {
        unsafe { (self.set_cursor_position)(self.this, context, x, y) }
    }
    pub fn is_topmost_enabled_context(&self, context: *mut InputContextT) -> bool {
        unsafe { (self.is_topmost_enabled_context)(self.this, context) }
    }
}
