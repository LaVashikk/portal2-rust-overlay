use std::ffi::{c_char, c_void, CStr};
use super::{CCommand, ConCommandBase, CvarFlags};

/// Represents a console command (`ConCommand`) in the Source engine (`portal2`).
///
/// A `ConCommand` is an executable command registered in the engine's console system (`ICvar`).
/// When invoked by the user or an engine script (e.g., `] my_command arg1 arg2`), the engine
/// dispatches execution to the registered `extern "C"` Rust function callback.
///
/// # Example
///
/// ```rust,no_run
/// use portal2_sdk::{ConCommand, CCommand, CvarFlags, con_print, con_color_print, Color};
///
/// extern "C" fn my_command_callback(cmd: &CCommand) {
///     con_print!("Command invoked! Total args: {}\n", cmd.arg_count());
///
///     if let Some(arg1) = cmd.arg(1) {
///         con_color_print!(Color::rgb(0, 255, 0), "Argument 1: {}\n", arg1);
///     }
/// }
///
/// // Registering the command right into the developer console:
/// ConCommand::register_new(
///     "my_test_cmd",
///     "Prints custom arguments to developer console",
///     CvarFlags::NONE,
///     my_command_callback,
/// ).expect("Failed to register ConCommand");
/// ```
#[repr(C)]
pub struct ConCommand {
    /// Base fields inherited from `ConCommandBase` (`vtable`, `name`, `help_string`, `flags`, `is_registered`).
    pub base: ConCommandBase,
    /// Pointer to the primary callback (`FnCommandCallback_t`), invoked when the command is dispatched.
    pub callback: Option<extern "C" fn(command: &CCommand)>, // +0x18
    /// Optional completion callback pointer (`FnCommandCompletionCallback`) used for auto-complete suggestions.
    pub completion_callback: *const c_void,                  // +0x1C
    /// Internal bitfields (`m_bHasCompletionCallback`, `m_bUsingNewCommandCallback`, `m_bUsingCommandCallbackInterface`).
    pub flags_and_bitfields: u32,                            // +0x20
}

/// A high-level builder (`ConCommandBuilder`) for constructing and registering custom `ConCommand`s cleanly.
///
/// # Example
///
/// ```rust,no_run
/// use portal2_sdk::{ConCommand, CCommand, CvarFlags};
///
/// extern "C" fn callback(cmd: &CCommand) {}
///
/// let cmd = ConCommand::builder("custom_reload", callback)
///     .help_text("Reloads configuration files")
///     .flags(CvarFlags::NONE)
///     .register()
///     .expect("Failed to register custom_reload");
/// ```
pub struct ConCommandBuilder<'a> {
    name: &'a str,
    help_string: &'a str,
    flags: CvarFlags,
    callback: extern "C" fn(cmd: &CCommand),
}

impl<'a> ConCommandBuilder<'a> {
    /// Creates a new `ConCommandBuilder` with the specified command name and execution callback.
    pub fn new(name: &'a str, callback: extern "C" fn(cmd: &CCommand)) -> Self {
        Self {
            name,
            help_string: "",
            flags: CvarFlags::NONE,
            callback,
        }
    }

    /// Sets the help description text displayed when users query `help <command>` or `find`.
    pub fn help_text(mut self, help_string: &'a str) -> Self {
        self.help_string = help_string;
        self
    }

    /// Sets the `CvarFlags` bitmask on this `ConCommand`.
    pub fn flags(mut self, flags: CvarFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Allocates and registers the `ConCommand` in the Source engine using vtables borrowed from an existing command.
    ///
    /// Returns `Some(&'static mut ConCommand)` on success, or `None` if the engine interface is unavailable.
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
    /// Returns a high-level builder (`ConCommandBuilder`) for configuring and registering a new console command.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use portal2_sdk::{ConCommand, CCommand};
    ///
    /// extern "C" fn my_callback(cmd: &CCommand) {}
    ///
    /// let cmd = ConCommand::builder("test_cmd", my_callback)
    ///     .help_text("Description of test_cmd")
    ///     .register();
    /// ```
    pub fn builder<'a>(
        name: &'a str,
        callback: extern "C" fn(cmd: &CCommand),
    ) -> ConCommandBuilder<'a> {
        ConCommandBuilder::new(name, callback)
    }

    /// High-level shortcut to register a simple `ConCommand` immediately without manual padding or pointers.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use portal2_sdk::{ConCommand, CCommand, CvarFlags};
    ///
    /// extern "C" fn survey_callback(cmd: &CCommand) {}
    ///
    /// ConCommand::register_new(
    ///     "open_survey",
    ///     "Target survey config path or 0 to close",
    ///     CvarFlags::NONE,
    ///     survey_callback,
    /// ).expect("Failed to register ConCommand");
    /// ```
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

    /// Checks if the `ConCommand` has been registered with the engine's `ICvar` system.
    pub fn is_registered(&self) -> bool {
        self.base.is_registered
    }

    /// Returns the name of the `ConCommand`.
    pub fn get_name(&self) -> &str {
        if self.base.name.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.base.name).to_str().unwrap_or("") }
    }

    /// Returns the help text description of the `ConCommand`.
    pub fn get_help_text(&self) -> &str {
        if self.base.help_string.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.base.help_string).to_str().unwrap_or("") }
    }
}
