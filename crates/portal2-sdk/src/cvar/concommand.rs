use std::ffi::{c_char, c_void, CStr};
use super::{CCommand, ConCommandBase, CvarFlags};

#[repr(C)]
pub struct ConCommand {
    // Inherits from ConCommandBase (Size: 0x18)
    pub base: ConCommandBase,
    pub callback: Option<extern "C" fn(command: &CCommand)>, // +0x18
    pub completion_callback: *const c_void,                  // +0x1C
    pub flags_and_bitfields: u32,                            // +0x20
}

/// A high-level builder for creating and registering custom `ConCommand`s cleanly.
pub struct ConCommandBuilder<'a> {
    name: &'a str,
    help_string: &'a str,
    flags: CvarFlags,
    callback: extern "C" fn(cmd: &CCommand),
}

impl<'a> ConCommandBuilder<'a> {
    pub fn new(name: &'a str, callback: extern "C" fn(cmd: &CCommand)) -> Self {
        Self {
            name,
            help_string: "",
            flags: CvarFlags::NONE,
            callback,
        }
    }

    pub fn help_text(mut self, help_string: &'a str) -> Self {
        self.help_string = help_string;
        self
    }

    pub fn flags(mut self, flags: CvarFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Allocates and registers the `ConCommand` in the Source engine using vtables borrowed from an existing command.
    pub fn register(self) -> Option<&'static mut ConCommand> {
        let engine = crate::get_engine();
        let cvar_system = engine.cvar_system();

        // Borrow vtable from a known valid ConCommand in the engine (e.g. "echo" or "help")
        let dummy = cvar_system
            .find_command_base("echo")
            .or_else(|| cvar_system.find_command_base("help"))?;
        let base_vtable = dummy.vtable;

        let name = std::ffi::CString::new(self.name).ok()?.into_raw();
        let help_string = std::ffi::CString::new(self.help_string).ok()?.into_raw();

        let command_box = Box::new(ConCommand {
            base: ConCommandBase {
                vtable: base_vtable,
                next: std::ptr::null_mut(),
                is_registered: false,
                _pad0: [0; 3],
                name,
                help_string,
                flags: self.flags.bits(),
            },
            callback: Some(self.callback),
            completion_callback: std::ptr::null(),
            // m_bHasCompletionCallback (bit 0) = 0
            // m_bUsingNewCommandCallback (bit 1) = 1 -> 0x02
            // m_bUsingCommandCallbackInterface (bit 2) = 0
            flags_and_bitfields: 2,
        });

        let command_ptr = Box::leak(command_box);

        cvar_system.register_con_command(&mut command_ptr.base);

        Some(command_ptr)
    }
}

impl ConCommand {
    /// Returns a high-level builder for configuring and registering a new console command (`ConCommand`).
    pub fn builder<'a>(
        name: &'a str,
        callback: extern "C" fn(cmd: &CCommand),
    ) -> ConCommandBuilder<'a> {
        ConCommandBuilder::new(name, callback)
    }

    /// High-level shortcut to register a simple ConCommand immediately without manual padding/pointers.
    pub fn register_new(
        name: &str,
        help_string: &str,
        flags: CvarFlags,
        callback: extern "C" fn(cmd: &CCommand),
    ) -> Option<&'static mut ConCommand> {
        ConCommandBuilder::new(name, callback)
            .help_text(help_string)
            .flags(flags)
            .register()
    }

    /// Checks if the ConCommand has been registered with the engine's CVar system.
    pub fn is_registered(&self) -> bool {
        self.base.is_registered
    }

    /// Returns the name of the ConCommand.
    pub fn get_name(&self) -> &str {
        if self.base.name.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.base.name).to_str().unwrap_or("") }
    }

    /// Returns the help text for the ConCommand.
    pub fn get_help_text(&self) -> &str {
        if self.base.help_string.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.base.help_string).to_str().unwrap_or("") }
    }
}
