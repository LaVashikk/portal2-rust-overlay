#![allow(unused)]
use std::ffi::{c_char, c_int, c_void, CStr};

mod flags;
mod convar;
mod concommand;
pub use flags::CvarFlags;
pub use convar::{ConVar, ConVarBuilder};
pub use concommand::{ConCommand, ConCommandBuilder};

/// RGBA Color structure used for colored developer console printing (`ConsoleColorPrintf`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
}

type FnSetValueStr = unsafe extern "thiscall" fn(this: *mut ConVar, value: *const c_char);
type FnSetValueFloat = unsafe extern "thiscall" fn(this: *mut ConVar, value: f32);
type FnSetValueInt = unsafe extern "thiscall" fn(this: *mut ConVar, value: i32);
type FnFindVar = unsafe extern "thiscall" fn(this: *mut RawICvar, var_name: *const c_char) -> *mut ConVar;
type FnFindCommandBase = unsafe extern "thiscall" fn(this: *mut RawICvar, name: *const c_char) -> *mut ConCommandBase;
type FnRegisterConCommand = unsafe extern "thiscall" fn(this: *mut RawICvar, base: *mut ConCommandBase);
type FnUnregisterConCommand = unsafe extern "thiscall" fn(this: *mut RawICvar, base: *mut ConCommandBase);
type FnConsoleColorPrintf = unsafe extern "C" fn(this: *mut RawICvar, color: *const Color, format: *const c_char, msg: *const c_char);
type FnConsolePrintf = unsafe extern "C" fn(this: *mut RawICvar, format: *const c_char, msg: *const c_char);

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
    pub vtable: *const ConVarVTable,
    pub next: *mut ConCommandBase,
    pub is_registered: bool,
    _pad0: [u8; 3],
    pub name: *const c_char,
    pub help_string: *const c_char,
    pub flags: c_int,
}

impl ConCommandBase {
    pub fn get_next<'a>(&mut self) -> Option<&'a mut ConCommandBase> {
        unsafe { self.next.as_mut() }
    }

    pub fn is_command(&self) -> bool {
        type FnIsCommand = unsafe extern "thiscall" fn(this: *const ConCommandBase) -> bool;
        unsafe {
            let vtable = *(self.vtable as *const *const usize);
            let func_ptr = vtable.add(1).read();
            let is_command_fn: FnIsCommand = std::mem::transmute(func_ptr);
            is_command_fn(self)
        }
    }

    pub fn is_registered(&self) -> bool {
        self.is_registered
    }

    pub fn get_name(&self) -> &str {
        if self.name.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.name).to_str().unwrap_or("") }
    }

    pub fn get_help_text(&self) -> &str {
        if self.help_string.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.help_string).to_str().unwrap_or("") }
    }
}

/// Represents the command arguments passed to a ConCommand callback.
#[repr(C)]
pub struct CCommand {
    pub argc: c_int,
    pub argv0_size: c_int,
    pub args_buffer: [c_char; 512],
    pub argv_buffer: [c_char; 512],
    pub argv: [*const c_char; 64],
    pub source: c_int,
}

impl CCommand {
    /// Returns the number of arguments (including the command name itself at index 0).
    pub fn arg_count(&self) -> usize {
        self.argc as usize
    }

    /// Legacy getter for argument count (`argc`).
    pub fn arg_c(&self) -> i32 {
        self.argc
    }

    /// Returns the argument at the given index, or `None` if the index is out of bounds.
    /// Index 0 is the command name itself (`argv[0]`), index 1 is the first argument (`argv[1]`).
    pub fn arg(&self, index: usize) -> Option<&str> {
        if index >= self.argc as usize || index >= 64 {
            return None;
        }
        unsafe {
            let ptr = self.argv[index];
            if ptr.is_null() {
                None
            } else {
                Some(CStr::from_ptr(ptr).to_str().unwrap_or(""))
            }
        }
    }

    /// Returns the argument at the given index as a string slice, or `""` if out of bounds or null.
    pub fn arg_str(&self, index: usize) -> &str {
        self.arg(index).unwrap_or("")
    }

    /// Returns the raw command string as typed by the user in console.
    pub fn command_string(&self) -> &str {
        if self.argc == 0 {
            return "";
        }
        unsafe {
            CStr::from_ptr(self.args_buffer.as_ptr()).to_str().unwrap_or("")
        }
    }
}

// Opaque type for the `this` pointer.
#[repr(C)] pub(crate) struct RawICvar { _private: [u8; 0] }

/// Represents an instance of the ICvar interface.
pub struct ICvar {
    pub(crate) this: *mut RawICvar,
    pub(crate) find_var: FnFindVar,
    pub(crate) find_command_base: FnFindCommandBase,
    pub(crate) register_con_command: FnRegisterConCommand,
    pub(crate) unregister_con_command: FnUnregisterConCommand,
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

    /// Finds any command base (ConVar or ConCommand) by name.
    pub fn find_command_base<'a>(&self, name: &str) -> Option<&'a mut ConCommandBase> {
        let c_name = match std::ffi::CString::new(name) {
            Ok(s) => s,
            Err(_) => return None,
        };
        unsafe {
            let base_ptr = (self.find_command_base)(self.this, c_name.as_ptr());
            base_ptr.as_mut()
        }
    }

    /// Registers a ConCommandBase (ConVar or ConCommand) with the engine's CVar system.
    pub fn register_con_command(&self, base: &mut ConCommandBase) {
        unsafe {
            (self.register_con_command)(self.this, base as *mut ConCommandBase);
        }
    }

    /// Unregisters a ConCommandBase (ConVar or ConCommand) from the engine's CVar system.
    pub fn unregister_con_command(&self, base: &mut ConCommandBase) {
        unsafe {
            (self.unregister_con_command)(self.this, base as *mut ConCommandBase);
        }
    }
}
