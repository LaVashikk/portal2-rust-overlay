#![allow(unused)]
use std::ffi::{c_char, c_int, c_void, CStr};

mod flags;
mod convar;
mod concommand;
pub use flags::CvarFlags;
pub use convar::{ConVar, ConVarBuilder};
pub use concommand::{ConCommand, ConCommandBuilder};

/// RGBA Color structure used for colored developer console printing via `ConsoleColorPrintf`.
///
/// # Example
///
/// ```rust,no_run
/// use portal2_sdk::Color;
///
/// let green = Color::rgb(0, 255, 0);
/// let yellow = Color::rgb(255, 200, 0);
/// let cyan_transparent = Color::rgba(0, 255, 255, 128);
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red color component (0-255).
    pub r: u8,
    /// Green color component (0-255).
    pub g: u8,
    /// Blue color component (0-255).
    pub b: u8,
    /// Alpha/opacity channel (0-255).
    pub a: u8,
}

impl Color {
    /// Standard pure white.
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    /// Standard pure black.
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    /// Soft red color , ideal for error messages inside the developer console.
    pub const RED: Self = Self::rgb(201, 74, 74);
    /// Bright pure red.
    pub const BRIGHT_RED: Self = Self::rgb(255, 60, 60);
    /// Soft green color (`#54C94A`), ideal for success/status messages.
    pub const GREEN: Self = Self::rgb(84, 201, 74);
    /// Bright pure green (`#00FF00`).
    pub const BRIGHT_GREEN: Self = Self::rgb(0, 255, 0);
    /// Soft orange color (`#EDA237`), ideal for warning messages.
    pub const ORANGE: Self = Self::rgb(237, 162, 55);
    /// Soft yellow color (`#EDE257`), ideal for highlights and attention.
    pub const YELLOW: Self = Self::rgb(237, 226, 87);
    /// Soft blue color (`#4A90C9`).
    pub const BLUE: Self = Self::rgb(74, 144, 201);
    /// Cyan color (`#4AC9C9`).
    pub const CYAN: Self = Self::rgb(74, 201, 201);
    /// Magenta / purple color (`#C94AC9`).
    pub const MAGENTA: Self = Self::rgb(201, 74, 201);
    /// Neutral gray / muted color (`#A0A0A0`), ideal for subtle or secondary logs.
    pub const GRAY: Self = Self::rgb(160, 160, 160);

    /// Constructs a new `Color` with explicit red, green, blue, and alpha components.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Constructs a new `Color` with red, green, and blue components, and fully opaque alpha (`255`).
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

/// Defines the virtual method table (`vtable`) for a `ConVar` object inheriting from `IConVar`.
#[repr(C)]
pub struct ConVarVTable {
    _pad0: [usize; 12],
    pub set_value_str: FnSetValueStr,
    pub set_value_float: FnSetValueFloat,
    pub set_value_int: FnSetValueInt,
}

/// The common base structure shared by `ConVar` and `ConCommand` inside the engine's `ICvar` registry.
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
    /// Returns a mutable reference to the next `ConCommandBase` in the linked list (`ICvar` chain).
    pub fn get_next<'a>(&mut self) -> Option<&'a mut ConCommandBase> {
        unsafe { self.next.as_mut() }
    }

    /// Checks if this `ConCommandBase` is an executable command (`ConCommand`) rather than a variable (`ConVar`).
    pub fn is_command(&self) -> bool {
        type FnIsCommand = unsafe extern "thiscall" fn(this: *const ConCommandBase) -> bool;
        unsafe {
            let vtable = *(self.vtable as *const *const usize);
            let func_ptr = vtable.add(1).read();
            let is_command_fn: FnIsCommand = std::mem::transmute(func_ptr);
            is_command_fn(self)
        }
    }

    /// Checks if this item is currently registered in the `ICvar` system.
    pub fn is_registered(&self) -> bool {
        self.is_registered
    }

    /// Returns the name of the command or variable as a string slice (`&str`).
    pub fn get_name(&self) -> &str {
        if self.name.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.name).to_str().unwrap_or("") }
    }

    /// Returns the help description associated with this command or variable.
    pub fn get_help_text(&self) -> &str {
        if self.help_string.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.help_string).to_str().unwrap_or("") }
    }
}

/// Represents the command arguments passed to a `ConCommand` execution callback.
///
/// Contains argument counts (`argc`), tokenized argument strings (`argv`), and the raw unparsed command buffer.
///
/// # Example
///
/// ```rust,no_run
/// use portal2_sdk::CCommand;
///
/// extern "C" fn my_callback(cmd: &CCommand) {
///     // Total arguments including command name at index 0:
///     let total_args = cmd.arg_count();
///
///     // Inspect first parameter safely with Option<&str>:
///     if let Some(param) = cmd.arg(1) {
///         con_print!("Received parameter: {}", param);
///     }
///
///     // Or get raw string slice without Option (returns "" if missing):
///     let param2 = cmd.arg_str(2);
/// }
/// ```
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
    /// Returns the number of arguments (including the command name itself at index `0`).
    pub fn arg_count(&self) -> usize {
        self.argc as usize
    }

    /// Legacy getter for argument count (`argc`).
    pub fn arg_c(&self) -> i32 {
        self.argc
    }

    /// Returns the argument at the given index as `Some(&str)`, or `None` if the index is out of bounds or null.
    ///
    /// - Index `0` is the command name itself (`argv[0]`).
    /// - Index `1` is the first parameter (`argv[1]`).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use portal2_sdk::CCommand;
    /// # let cmd: &CCommand = unsafe { std::mem::zeroed() };
    /// match cmd.arg(1) {
    ///     Some("on") => con_print!("Enabled"),
    ///     Some("off") => con_print!("Disabled"),
    ///     _ => con_print!("Usage: my_command <on|off>"),
    /// }
    /// ```
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

    /// Returns the argument at the given index as a string slice (`&str`), returning `""` if out of bounds or null.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use portal2_sdk::CCommand;
    /// # let cmd: &CCommand = unsafe { std::mem::zeroed() };
    /// if cmd.arg_str(1) == "force" {
    ///     // Execute force mode
    /// }
    /// ```
    pub fn arg_str(&self, index: usize) -> &str {
        self.arg(index).unwrap_or("")
    }

    /// Returns the entire raw command string exactly as typed by the user in the developer console.
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

/// Represents an instance of the `ICvar` interface (the central console subsystem in Portal 2).
///
/// Provides methods for finding existing console variables (`ConVar`) and commands (`ConCommand`),
/// registering new custom variables/commands, and printing messages directly to the developer console (`~`).
///
/// # Example
///
/// ```rust,no_run
/// use portal2_sdk::{Color, con_print, con_color_print};
///
/// let engine = portal2_sdk::get_engine();
/// let cvar_system = engine.cvar_system();
///
/// // Find and modify an existing engine cvar:
/// if let Some(cheats) = cvar_system.find_var("sv_cheats") {
///     cheats.set_value_int(1);
/// }
///
/// // Print directly to the developer console:
/// cvar_system.console_print("Hello from Rust SDK!\n");
/// cvar_system.console_color_print(Color::rgb(0, 255, 0), "[SUCCESS] Mod initialized!\n");
/// ```
pub struct ICvar {
    pub(crate) this: *mut RawICvar,
    pub(crate) find_var: FnFindVar,
    pub(crate) find_command_base: FnFindCommandBase,
    pub(crate) register_con_command: FnRegisterConCommand,
    pub(crate) unregister_con_command: FnUnregisterConCommand,
    pub(crate) console_color_printf: FnConsoleColorPrintf,
    pub(crate) console_printf: FnConsolePrintf,
}

impl ICvar {
    /// Finds an existing console variable (`ConVar`) by exact name.
    ///
    /// Returns `Some(&mut ConVar)` if found, or `None` if no variable exists with that name.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # let cvar_system = portal2_sdk::get_engine().cvar_system();
    /// if let Some(timescale) = cvar_system.find_var("host_timescale") {
    ///     timescale.set_value_float(0.5); // Slow motion
    /// }
    /// ```
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

    /// Finds any command base (`ConCommandBase`), which could be either a `ConVar` or `ConCommand`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # let cvar_system = portal2_sdk::get_engine().cvar_system();
    /// if let Some(base) = cvar_system.find_command_base("echo") {
    ///     con_print!("Command/Var exists: {}", base.get_name());
    /// }
    /// ```
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

    /// Registers a custom `ConCommandBase` (`ConVar` or `ConCommand`) into the engine's `ICvar` registry.
    ///
    /// Note: Generally you should prefer using high-level builders `ConVar::builder()` or `ConCommand::builder()`
    /// which invoke this method automatically.
    pub fn register_con_command(&self, base: &mut ConCommandBase) {
        unsafe {
            (self.register_con_command)(self.this, base as *mut ConCommandBase);
        }
    }

    /// Unregisters and removes a `ConCommandBase` (`ConVar` or `ConCommand`) from the engine's `ICvar` registry.
    pub fn unregister_con_command(&self, base: &mut ConCommandBase) {
        unsafe {
            (self.unregister_con_command)(self.this, base as *mut ConCommandBase);
        }
    }

    /// Prints standard text directly to the in-game developer console (`~`).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # let cvar_system = portal2_sdk::get_engine().cvar_system();
    /// cvar_system.console_print("Initializing plugin systems...\n");
    /// ```
    pub fn console_print(&self, msg: &str) {
        if let Ok(c_msg) = std::ffi::CString::new(msg) {
            let fmt = b"%s\0".as_ptr() as *const c_char;
            unsafe {
                (self.console_printf)(self.this, fmt, c_msg.as_ptr());
            }
        }
    }

    /// Prints colored text (`RGBA`) directly to the in-game developer console (`~`).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use portal2_sdk::Color;
    /// # let cvar_system = portal2_sdk::get_engine().cvar_system();
    /// cvar_system.console_color_print(
    ///     Color::rgb(0, 255, 0),
    ///     "[SUCCESS] Portal 2 Playtest Tool loaded!\n"
    /// );
    /// ```
    pub fn console_color_print(&self, color: Color, msg: &str) {
        if let Ok(c_msg) = std::ffi::CString::new(msg) {
            let fmt = b"%s\0".as_ptr() as *const c_char;
            unsafe {
                (self.console_color_printf)(self.this, &color as *const Color, fmt, c_msg.as_ptr());
            }
        }
    }
}
