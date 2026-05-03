use std::ffi::{c_char, c_int, CString};
use crate::types::{Vector, QAngle};

// Opaque type for the `this` pointer.
#[repr(C)] pub(crate) struct RawIVDebugOverlay { _private: [u8; 0] }

type FnAddBoxOverlay = unsafe extern "thiscall" fn(this: *mut RawIVDebugOverlay, origin: *const Vector, mins: *const Vector, max: *const Vector, orientation: *const QAngle, r: c_int, g: c_int, b: c_int, a: c_int, duration: f32);
type FnAddSphereOverlay = unsafe extern "thiscall" fn(this: *mut RawIVDebugOverlay, origin: *const Vector, radius: f32, theta: c_int, phi: c_int, r: c_int, g: c_int, b: c_int, a: c_int, duration: f32);
type FnAddLineOverlay = unsafe extern "thiscall" fn(this: *mut RawIVDebugOverlay, start: *const Vector, end: *const Vector, r: c_int, g: c_int, b: c_int, no_depth_test: bool, duration: f32);
type FnAddTextOverlay = unsafe extern "C" fn(this: *mut RawIVDebugOverlay, origin: *const Vector, duration: f32, format: *const c_char, ...);
type FnAddScreenTextOverlay = unsafe extern "thiscall" fn(this: *mut RawIVDebugOverlay, x: f32, y: f32, duration: f32, r: c_int, g: c_int, b: c_int, a: c_int, text: *const c_char);
type FnScreenPosition = unsafe extern "thiscall" fn(this: *mut RawIVDebugOverlay, point: *const Vector, screen: *mut Vector) -> c_int;
type FnClearAllOverlays = unsafe extern "thiscall" fn(this: *mut RawIVDebugOverlay);

/// Interface for drawing debug shapes and text in the 3D world.
pub struct IVDebugOverlay {
    pub(crate) this: *mut RawIVDebugOverlay,

    pub(crate) add_box_overlay: FnAddBoxOverlay,
    pub(crate) add_sphere_overlay: FnAddSphereOverlay,
    pub(crate) add_line_overlay: FnAddLineOverlay,
    pub(crate) add_text_overlay: FnAddTextOverlay,
    pub(crate) add_screen_text_overlay: FnAddScreenTextOverlay,
    pub(crate) screen_position: FnScreenPosition,
    pub(crate) clear_all_overlays: FnClearAllOverlays,
}

impl IVDebugOverlay {
    /// Adds a 3D box overlay.
    pub fn add_box_overlay(&self, origin: &Vector, mins: &Vector, maxs: &Vector, angles: &QAngle, r: i32, g: i32, b: i32, a: i32, duration: f32) {
        unsafe { (self.add_box_overlay)(self.this, origin, mins, maxs, angles, r as c_int, g as c_int, b as c_int, a as c_int, duration) }
    }

    /// Adds a 3D sphere overlay.
    pub fn add_sphere_overlay(&self, origin: &Vector, radius: f32, theta: i32, phi: i32, r: i32, g: i32, b: i32, a: i32, duration: f32) {
        unsafe { (self.add_sphere_overlay)(self.this, origin, radius, theta as c_int, phi as c_int, r as c_int, g as c_int, b as c_int, a as c_int, duration) }
    }

    /// Adds a line overlay between two points.
    pub fn add_line_overlay(&self, start: &Vector, end: &Vector, r: i32, g: i32, b: i32, no_depth_test: bool, duration: f32) {
        unsafe { (self.add_line_overlay)(self.this, start, end, r as c_int, g as c_int, b as c_int, no_depth_test, duration) }
    }

    /// Adds text in the 3D world at the specified origin.
    pub fn add_text_overlay(&self, origin: &Vector, duration: f32, text: &str) {
        if let Ok(c_str) = CString::new(text) {
            let fmt = b"%s\0".as_ptr() as *const c_char;
            unsafe { (self.add_text_overlay)(self.this, origin, duration, fmt, c_str.as_ptr()) }
        }
    }

    /// Adds text directly to the screen. Coordinates are normalized (0.0 to 1.0) or pixels depending on context.
    pub fn add_screen_text_overlay(&self, x: f32, y: f32, duration: f32, r: i32, g: i32, b: i32, a: i32, text: &str) {
        if let Ok(c_str) = CString::new(text) {
            unsafe { (self.add_screen_text_overlay)(self.this, x, y, duration, r as c_int, g as c_int, b as c_int, a as c_int, c_str.as_ptr()) }
        }
    }

    /// Converts a 3D world position to a 2D screen position.
    /// Returns `true` if the point is on screen, `false` if it is culled or behind the camera.
    pub fn world_to_screen(&self, point: &Vector) -> Option<Vector> {
        let mut screen = Vector::default();
        // ScreenPosition returns 1 if the point is clipped/behind, 0 if successful.
        let result = unsafe { (self.screen_position)(self.this, point, &mut screen) };
        if result == 0 { Some(screen) } else { None }
    }

    /// Removes all active overlays.
    pub fn clear_all_overlays(&self) {
        unsafe { (self.clear_all_overlays)(self.this) }
    }
}
