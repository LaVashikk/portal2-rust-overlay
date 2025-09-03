use std::ffi::{c_char, c_int, c_void};

#[repr(C)]
pub struct ConCommandBase {
    vtable: *const c_void,      // +0x00 (size 4)
    next: *mut ConCommandBase,  // +0x04 (size 4)
    is_registered: bool,        // +0x08 (size 1)
    _pad0: [u8; 3],             // +0x09 (size 3, alignment)
    name: *const c_char,        // +0x0C (size 4)
    help_string: *const c_char, // +0x10 (size 4)
    flags: c_int,               // +0x14 (size 4)
}

#[repr(C)]
pub struct ConVar {
    // Inherits from ConCommandBase
    base: ConCommandBase,

    iconvar_vtable: *const c_void,
    parent: *mut ConVar,          // Offset 0x1C (28)
    default_value: *const c_char, // Offset 0x20 (32)
    string: *mut c_char,  // Offset 0x24 (36)
    string_length: c_int, // Offset 0x28 (40)
    float_value: f32,     // Offset 0x2C (44)
    int_value: i32,       // Offset 0x30 (48)
}

impl ConVar {
    pub fn get_int(&self) -> i32 {
        self.int_value
    }

    /// Returns the float value of this [`ConVar`].
    pub fn get_float(&self) -> f32 {
        self.float_value
    }

    pub fn get_string(&self) -> String {
        unsafe {
            if self.string.is_null() {
                return String::new();
            }
            std::ffi::CStr::from_ptr(self.string)
                .to_string_lossy()
                .into_owned()
        }
    }
}

// Opaque type for the `this` pointer.
#[repr(C)] pub(crate) struct RawICvar { _private: [u8; 0] }

type FnFindVar = unsafe extern "thiscall" fn(this: *mut RawICvar, var_name: *const c_char) -> *mut ConVar;

/// Represents an instance of the ICvar interface.
pub struct ICvar {
    pub(crate) this: *mut RawICvar,
    pub(crate) find_var: FnFindVar,
}

impl ICvar {
    /// Finds a console variable by name.
    pub fn find_var<'a>(&self, name: &str) -> Option<&'a ConVar> {
        let c_name = match std::ffi::CString::new(name) {
            Ok(s) => s,
            Err(_) => return None,
        };
        unsafe {
            let convar_ptr = (self.find_var)(self.this, c_name.as_ptr());
            convar_ptr.as_ref()
        }
    }
}
