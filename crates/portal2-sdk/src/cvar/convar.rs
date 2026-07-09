use std::ffi::{c_char, c_int, c_void, CStr};

use super::{ConCommandBase, CvarFlags};

/// Represents a console variable (`ConVar`) in the Source engine (`portal2`).
///
/// A `ConVar` stores floating-point, integer, and string representations of a setting, along with
/// optional min/max bounds and flags (`CvarFlags`) that govern archival (`FCVAR_ARCHIVE`), cheat
/// protection (`FCVAR_CHEAT`), replication (`FCVAR_REPLICATED`), etc.
///
/// # Example
///
/// ```rust,no_run
/// use portal2_sdk::{ConVar, CvarFlags};
///
/// // Quick registration of a new ConVar:
/// let my_cvar = ConVar::register_new(
///     "my_custom_setting",
///     "100",
///     "Help description for my setting",
///     CvarFlags::ARCHIVE,
/// ).expect("Failed to register ConVar");
///
/// // Reading values:
/// let int_val = my_cvar.get_int();
/// let float_val = my_cvar.get_float();
/// let str_val = my_cvar.get_string();
///
/// // Modifying value natively:
/// my_cvar.set_value_int(200);
/// ```
#[repr(C)]
pub struct ConVar {
    /// Base fields inherited from `ConCommandBase` (`vtable`, `name`, `help_string`, `flags`, `is_registered`).
    pub base: ConCommandBase,            // Size: 0x18

    pub iconvar_vtable: *const c_void,
    pub parent: *mut ConVar,             // +0x18
    pub default_value: *const c_char,    // +0x1C
    pub string: *mut c_char,             // +0x20
    pub string_length: c_int,            // +0x24
    pub float_value: f32,                // +0x28
    pub int_value: i32,                  // +0x2C
    pub has_min: bool,                   // +0x30

    pub _pad_min: [u8; 3],               // +0x31
    pub min_val: f32,                    // +0x34
    pub has_max: bool,                   // +0x38
    pub _pad_max: [u8; 3],               // +0x39

    pub max_val: f32,                    // +0x3C
    pub change_callback: *const c_void,  // +0x40
}

/// A high-level builder for constructing and registering new custom `ConVar`s cleanly.
///
/// Uses the builder pattern to configure optional bounds (`min`/`max`), help strings, and flags
/// before allocating and inserting the variable into the engine's global `ICvar` registry.
///
/// # Example
///
/// ```rust,no_run
/// use portal2_sdk::{ConVar, CvarFlags};
///
/// let cvar = ConVar::builder("fov_custom_limit", "90.0")
///     .help_text("Custom Field of View limit")
///     .flags(CvarFlags::ARCHIVE | CvarFlags::CHEAT)
///     .min(60.0)
///     .max(140.0)
///     .register()
///     .expect("Failed to register bounded ConVar");
/// ```
pub struct ConVarBuilder<'a> {
    name: &'a str,
    default_value: &'a str,
    help_string: &'a str,
    flags: CvarFlags,
    min: Option<f32>,
    max: Option<f32>,
}

impl<'a> ConVarBuilder<'a> {
    /// Creates a new `ConVarBuilder` with the specified name and initial/default value.
    pub fn new(name: &'a str, default_value: &'a str) -> Self {
        Self {
            name,
            default_value,
            help_string: "",
            flags: CvarFlags::NONE,
            min: None,
            max: None,
        }
    }

    /// Sets the help text displayed when users query the `ConVar` using `help <cvar>` or `find`.
    pub fn help_text(mut self, help_string: &'a str) -> Self {
        self.help_string = help_string;
        self
    }

    /// Sets the engine `CvarFlags` bitmask on this `ConVar`.
    pub fn flags(mut self, flags: CvarFlags) -> Self {
        self.flags = flags;
        self
    }

    /// Sets the minimum allowed floating-point value for this `ConVar`.
    pub fn min(mut self, min_val: f32) -> Self {
        self.min = Some(min_val);
        self
    }

    /// Sets the maximum allowed floating-point value for this `ConVar`.
    pub fn max(mut self, max_val: f32) -> Self {
        self.max = Some(max_val);
        self
    }

    /// Allocates and registers the `ConVar` inside the Source engine (`ICvar`).
    ///
    /// Returns `Some(&'static mut ConVar)` upon success, or `None` if engine interfaces aren't initialized.
    pub fn register(self) -> Option<&'static mut ConVar> {
        let engine = crate::get_engine();
        let cvar_system = engine.cvar_system();

        // Borrow vtables from a known valid cvar in the engine
        let dummy = cvar_system.find_var("sv_cheats")?;
        let base_vtable = dummy.base.vtable;
        let dummy_raw = dummy as *const ConVar as *const *const c_void;
        let iconvar_vtable = unsafe { dummy_raw.add(6).read() }; // offset +0x18 / 4 = 6

        let name = std::ffi::CString::new(self.name).ok()?.into_raw();
        let help_string = std::ffi::CString::new(self.help_string).ok()?.into_raw();
        let default_val = std::ffi::CString::new(self.default_value).ok()?.into_raw();
        let string_buf = std::ffi::CString::new(self.default_value).ok()?.into_raw();

        let float_val = self.default_value.parse::<f32>().unwrap_or(0.0);
        let int_val = self.default_value.parse::<i32>().unwrap_or(0);

        let (has_min, min_val) = match self.min {
            Some(v) => (true, v),
            None => (false, 0.0),
        };
        let (has_max, max_val) = match self.max {
            Some(v) => (true, v),
            None => (false, 0.0),
        };

        let convar_box = Box::new(ConVar {
            base: ConCommandBase {
                vtable: base_vtable,
                next: std::ptr::null_mut(),
                is_registered: false,
                _pad0: [0; 3],
                name,
                help_string,
                flags: self.flags.bits(),
            },
            iconvar_vtable,
            parent: std::ptr::null_mut(),
            default_value: default_val,
            string: string_buf,
            string_length: (self.default_value.len() + 1) as c_int,
            float_value: float_val,
            int_value: int_val,
            has_min,
            _pad_min: [0; 3],
            min_val,
            has_max,
            _pad_max: [0; 3],
            max_val,
            change_callback: std::ptr::null(),
        });

        let convar_ptr = Box::leak(convar_box);
        convar_ptr.parent = convar_ptr as *mut ConVar;

        cvar_system.register_con_command(&mut convar_ptr.base);

        Some(convar_ptr)
    }
}

impl ConVar {
    /// Returns a high-level builder (`ConVarBuilder`) for configuring and registering a new console variable.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use portal2_sdk::{ConVar, CvarFlags};
    ///
    /// let cvar = ConVar::builder("test_cvar", "1")
    ///     .help_text("Test custom setting")
    ///     .flags(CvarFlags::NONE)
    ///     .register();
    /// ```
    pub fn builder<'a>(name: &'a str, default_value: &'a str) -> ConVarBuilder<'a> {
        ConVarBuilder::new(name, default_value)
    }

    /// High-level shortcut to register a simple `ConVar` immediately without manual padding or pointers.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use portal2_sdk::{ConVar, CvarFlags};
    ///
    /// let cvar = ConVar::register_new(
    ///     "open_survey",
    ///     "0",
    ///     "Target survey config path or 0 to close",
    ///     CvarFlags::NONE,
    /// ).expect("Failed to register ConVar");
    /// ```
    pub fn register_new(
        name: &str,
        default_value: &str,
        help_string: &str,
        flags: CvarFlags,
    ) -> Option<&'static mut ConVar> {
        ConVarBuilder::new(name, default_value)
            .help_text(help_string)
            .flags(flags)
            .register()
    }

    fn vtable(&self) -> &super::ConVarVTable {
        unsafe { &*self.base.vtable }
    }

    /// Returns the current value of the `ConVar` as a 32-bit signed integer (`i32`).
    pub fn get_int(&self) -> i32 {
        self.int_value
    }

    /// Returns the current value of the `ConVar` as a 32-bit floating-point number (`f32`).
    pub fn get_float(&self) -> f32 {
        self.float_value
    }

    /// Returns the current value of the `ConVar` as a newly allocated Rust `String`.
    /// Returns an empty string if the internal string buffer pointer is null.
    pub fn get_string(&self) -> String {
        unsafe {
            if self.string.is_null() {
                return String::new();
            }
            std::ffi::CStr::from_ptr(self.string).to_string_lossy().into_owned()
        }
    }

    /// Returns the current value of the `ConVar` as a boolean (`true` if `get_int() != 0`).
    pub fn get_bool(&self) -> bool {
        self.get_int() != 0
    }

    /// Returns the current `CvarFlags` bitmask associated with this `ConVar`.
    pub fn get_flags(&self) -> CvarFlags {
        CvarFlags::from_bits_truncate(self.base.flags)
    }

    /// Adds one or more flags (`CvarFlags`) to the `ConVar`'s existing bitmask.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use portal2_sdk::CvarFlags;
    /// # let mut cvar = portal2_sdk::get_engine().cvar_system().find_var("sv_cheats").unwrap();
    /// cvar.add_flags(CvarFlags::ARCHIVE);
    /// ```
    pub fn add_flags(&mut self, flags_to_add: CvarFlags) {
        self.base.flags |= flags_to_add.bits();
    }

    /// Removes one or more flags (`CvarFlags`) from the `ConVar`'s existing bitmask.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use portal2_sdk::CvarFlags;
    /// # let mut cvar = portal2_sdk::get_engine().cvar_system().find_var("sv_cheats").unwrap();
    /// // Strip cheat flag so clients can modify it freely:
    /// cvar.remove_flags(CvarFlags::CHEAT);
    /// ```
    pub fn remove_flags(&mut self, flags_to_remove: CvarFlags) {
        self.base.flags &= !flags_to_remove.bits();
    }

    /// Checks whether a specific `CvarFlags` flag is set on this `ConVar`.
    pub fn is_flag_set(&self, flag: CvarFlags) -> bool {
        self.get_flags().contains(flag)
    }

    /// Checks if the `ConVar` has been successfully registered with the engine's `ICvar` system.
    pub fn is_registered(&self) -> bool {
        self.base.is_registered
    }

    /// Returns the initial/default value of the `ConVar` as a Rust `String`.
    pub fn get_default(&self) -> String {
        unsafe {
            if self.default_value.is_null() {
                return String::new();
            }
            CStr::from_ptr(self.default_value).to_string_lossy().into_owned()
        }
    }

    /// Returns the name of the `ConVar`.
    pub fn get_name(&self) -> &str {
        if self.base.name.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.base.name).to_str().unwrap_or("") }
    }

    /// Returns the help description text of the `ConVar`.
    pub fn get_help_text(&self) -> &str {
        if self.base.help_string.is_null() {
            return "";
        }
        unsafe { CStr::from_ptr(self.base.help_string).to_str().unwrap_or("") }
    }

    /// Returns the minimum allowed value as `Some(f32)` if bounds checking is enabled, or `None`.
    pub fn get_min(&self) -> Option<f32> {
        if self.has_min { Some(self.min_val) } else { None }
    }

    /// Returns the maximum allowed value as `Some(f32)` if bounds checking is enabled, or `None`.
    pub fn get_max(&self) -> Option<f32> {
        if self.has_max { Some(self.max_val) } else { None }
    }

    /// Sets the ConVar value using a string, correctly invoking engine callbacks.
    pub fn set_value_str(&mut self, value: &str) {
        if let Ok(c_str) = std::ffi::CString::new(value) {
            unsafe { (self.vtable().set_value_str)(self, c_str.as_ptr()) };
        }
    }

    /// Sets the `ConVar` value using a float, correctly invoking engine callbacks.
    pub fn set_value_float(&mut self, value: f32) {
        unsafe { (self.vtable().set_value_float)(self, value) };
    }

    /// Sets the ConVar value using an integer, correctly invoking engine callbacks.
    pub fn set_value_int(&mut self, value: i32) {
        unsafe { (self.vtable().set_value_int)(self, value) };
    }

    /// Resets the `ConVar` back to its default value (`default_value`).
    pub fn reset(&mut self) {
        let default_str = self.get_default();

        if let Ok(int_val) = default_str.parse::<i32>() {
            self.set_value_int(int_val);
        }

        if let Ok(float_val) = default_str.parse::<f32>() {
            self.set_value_float(float_val);
        }

        self.set_value_str(&default_str);
    }
}
