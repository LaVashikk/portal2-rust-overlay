use std::ffi::{c_char, c_int, c_void, CStr};

use super::{ConCommandBase, CvarFlags};

#[repr(C)]
pub struct ConVar {
    // Inherits from ConCommandBase
    pub base: ConCommandBase,            // Size: 0x18

    iconvar_vtable: *const c_void,
    parent: *mut ConVar,                 // +0x18
    default_value: *const c_char,        // +0x1C
    string: *mut c_char,                 // +0x20
    string_length: c_int,                // +0x24
    float_value: f32,                    // +0x28
    int_value: i32,                      // +0x2C
    has_min: bool,                       // +0x30

    _pad_min: [u8; 3],                   // +0x31
    min_val: f32,                        // +0x34
    has_max: bool,                       // +0x38
    _pad_max: [u8; 3],                   // +0x39

    max_val: f32,                        // +0x3C
    change_callback: *const c_void,      // +0x40
}

impl ConVar {
    fn vtable(&self) -> &super::ConVarVTable {
        unsafe { &*self.base.vtable }
    }

    /// Returns the ConVar's value as an integer.
    pub fn get_int(&self) -> i32 {
        self.int_value
    }

    /// Returns the ConVar's value as a float.
    pub fn get_float(&self) -> f32 {
        self.float_value
    }

    /// Returns the ConVar's value as a string.
    /// Returns an empty string if the pointer is null.
    pub fn get_string(&self) -> String {
        unsafe {
            if self.string.is_null() {
                return String::new();
            }
            std::ffi::CStr::from_ptr(self.string).to_string_lossy().into_owned()
        }
    }

    /// Returns the ConVar's value as a boolean.
    pub fn get_bool(&self) -> bool {
        self.get_int() != 0
    }

    /// Returns the ConVar's flags as a `CvarFlags` bitmask.
    pub fn get_flags(&self) -> CvarFlags {
        CvarFlags::from_bits_truncate(self.base.flags)
    }

    /// Adds one or more flags to the ConVar's existing flags.
    pub fn add_flags(&mut self, flags_to_add: CvarFlags) {
        self.base.flags |= flags_to_add.bits();
    }

    /// Removes one or more flags from the ConVar's existing flags.
    pub fn remove_flags(&mut self, flags_to_remove: CvarFlags) {
        self.base.flags &= !flags_to_remove.bits();
    }

    /// Checks if a specific flag is set on the ConVar.
    pub fn is_flag_set(&self, flag: CvarFlags) -> bool {
        self.get_flags().contains(flag)
    }

    /// Checks if the ConVar has been registered with the engine's CVar system.
    pub fn is_registered(&self) -> bool {
        self.base.is_registered
    }

    /// Returns the default value of the ConVar as a string.
    pub fn get_default(&self) -> String {
        unsafe {
            if self.default_value.is_null() {
                return String::new();
            }
            CStr::from_ptr(self.default_value).to_string_lossy().into_owned()
        }
    }

    /// Returns the name of the ConVar.
    pub fn get_name(&self) -> &str {
        if self.base.name.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.base.name).to_str().unwrap_or("") }
    }

    /// Returns the help text for the ConVar.
    pub fn get_help_text(&self) -> &str {
        if self.base.help_string.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.base.help_string).to_str().unwrap_or("") }
    }

    /// Returns the minimum allowed value, if one is set.
    pub fn get_min(&self) -> Option<f32> {
        if self.has_min { Some(self.min_val) } else { None }
    }

    /// Returns the maximum allowed value, if one is set.
    pub fn get_max(&self) -> Option<f32> {
        if self.has_max { Some(self.max_val) } else { None }
    }

    /// Sets the ConVar value using a string, correctly invoking engine callbacks.
    pub fn set_value_str(&mut self, value: &str) {
        if let Ok(c_str) = std::ffi::CString::new(value) {
            unsafe { (self.vtable().set_value_str)(self, c_str.as_ptr()) };
        }
    }

    /// Sets the ConVar value using a float, correctly invoking engine callbacks.
    pub fn set_value_float(&mut self, value: f32) {
        unsafe { (self.vtable().set_value_float)(self, value) };
    }

    /// Sets the ConVar value using an integer, correctly invoking engine callbacks.
    pub fn set_value_int(&mut self, value: i32) {
        unsafe { (self.vtable().set_value_int)(self, value) };
    }
}
