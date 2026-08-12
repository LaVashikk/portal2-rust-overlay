//! Platform abstraction over the dynamic loader.
//!
//! Everything OS-specific in this crate lives here: locating the engine's loaded
//! modules, pulling the `CreateInterface` factory out of them, and figuring out
//! where our own binary sits on disk. The rest of the SDK talks to the engine
//! through plain pointers and is platform-agnostic.
//!
//! The Windows build (the one everyone actually plays, natively or through Proton)
//! is fully supported. The native Linux build is a work in progress - this layer
//! already works there, but [`crate::Engine::initialize`] does not: it still
//! resolves functions through Windows-only byte signatures and MSVC vtable layouts.
//!
//! Both builds of Portal 2 are 32-bit; the structure layouts in [`crate::types`]
//! assume `target_pointer_width = "32"`.

pub(crate) mod abi;

#[cfg_attr(target_os = "windows", path = "windows.rs")]
#[cfg_attr(target_os = "linux", path = "linux.rs")]
mod imp;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
compile_error!("portal2-sdk supports the Windows and native Linux builds of Portal 2 only");

use std::ffi::CStr;
use std::path::PathBuf;

/// An engine module the SDK resolves interfaces from.
///
/// The Windows build ships these as DLLs, the native Linux build as shared objects
/// with the same base name (`engine.dll` / `engine.so`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Module {
    /// `engine` - the core engine: client state, events, tracing, debug overlay.
    Engine,
    /// `inputsystem` - mouse/keyboard capture and input contexts.
    InputSystem,
    /// `vstdlib` - Valve's runtime library; hosts the cvar system.
    VStdLib,
    /// `server` - the server-side game DLL, loaded only once a map is running.
    Server,
}

impl Module {
    /// The module's file name as the dynamic loader knows it.
    #[cfg(target_os = "windows")]
    pub const fn file_name(self) -> &'static CStr {
        match self {
            Self::Engine => c"engine.dll",
            Self::InputSystem => c"inputsystem.dll",
            Self::VStdLib => c"vstdlib.dll",
            Self::Server => c"server.dll",
        }
    }

    /// The module's file name as the dynamic loader knows it.
    #[cfg(target_os = "linux")]
    pub const fn file_name(self) -> &'static CStr {
        match self {
            Self::Engine => c"engine.so",
            Self::InputSystem => c"inputsystem.so",
            Self::VStdLib => c"vstdlib.so",
            Self::Server => c"server.so",
        }
    }
}

/// The address range a module's code is mapped at, as `(base, size)`.
///
/// Intended for signature scanning, so it covers the executable part of the module
/// and nothing else - do not treat it as the full mapped image.
///
/// Returns `None` if the module is not currently loaded. `server` in particular is
/// absent until a map has been loaded.
pub fn module_range(module: Module) -> Option<(*const u8, usize)> {
    imp::module_range(module)
}

/// Requests an interface from a module's `CreateInterface` factory.
///
/// This is how every engine interface is obtained: the module exports a single
/// `CreateInterface` symbol that maps a versioned name to a pointer to the
/// singleton implementing it.
///
/// Returns a null pointer if the module is not loaded, exports no factory, or does
/// not know the requested interface name.
///
/// # Safety
///
/// The returned pointer is only meaningful when interpreted as the interface that
/// was actually asked for; the engine performs no type checking beyond the name.
pub unsafe fn find_interface<T>(module: Module, interface_name: &CStr) -> *mut T {
    unsafe { imp::find_interface(module, interface_name) }
}

/// The directory containing the binary this code is running from.
///
/// Exposed to users through [`crate::utils::get_dll_directory`].
pub(crate) fn module_dir_of_self() -> Option<PathBuf> {
    imp::module_dir_of_self()
}
