#![allow(unused)]
use std::ffi::{c_char, c_int, c_void, CStr};

mod flags;
mod convar;
pub use flags::CvarFlags;
pub use convar::ConVar;

type FnSetValueStr = unsafe extern "thiscall" fn(this: *mut ConVar, value: *const c_char);
type FnSetValueFloat = unsafe extern "thiscall" fn(this: *mut ConVar, value: f32);
type FnSetValueInt = unsafe extern "thiscall" fn(this: *mut ConVar, value: i32);
type FnFindVar = unsafe extern "thiscall" fn(this: *mut RawICvar, var_name: *const c_char) -> *mut ConVar;

/// Defines the virtual method table for a ConVar object, which inherits from IConVar.
#[repr(C)]
pub struct ConVarVTable {
    _pad0: [usize; 12],
    pub set_value_str: FnSetValueStr,
    pub set_value_float: FnSetValueFloat,
    pub set_value_int: FnSetValueInt,
}

#[repr(C)]
pub struct ConCommandBase {
    vtable: *const ConVarVTable,
    next: *mut ConCommandBase,
    is_registered: bool,
    _pad0: [u8; 3],
    pub name: *const c_char,
    pub help_string: *const c_char,
    flags: c_int,
}

impl ConCommandBase {
    pub fn get_next<'a>(&mut self) -> Option<&'a mut ConCommandBase> {
        unsafe { self.next.as_mut() }
    }

    pub fn is_command(&self) -> bool { // todo!
        type FnIsCommand = unsafe extern "thiscall" fn(this: *const ConCommandBase) -> bool;
        unsafe {
            let vtable = *(self.vtable as *const *const usize);
            let func_ptr = vtable.add(1).read();
            let is_command_fn: FnIsCommand = std::mem::transmute(func_ptr);
            is_command_fn(self)
        }
    }
}

// Opaque type for the `this` pointer.
#[repr(C)] pub(crate) struct RawICvar { _private: [u8; 0] }

/// Represents an instance of the ICvar interface.
pub struct ICvar {
    pub(crate) this: *mut RawICvar,
    pub(crate) find_var: FnFindVar,
}

impl ICvar {
    /// Finds a console variable by name.
    pub fn find_var<'a>(&self, name: &str) -> Option<&'a mut ConVar> {
        let c_name = match std::ffi::CString::new(name) {
            Ok(s) => s,
            Err(_) => return None,
        };
        unsafe {
            let convar_ptr = (self.find_var)(self.this, c_name.as_ptr());
            convar_ptr.as_mut()
        }
    }
}
