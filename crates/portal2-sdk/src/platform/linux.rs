//! Linux implementation of the loader abstraction, on top of glibc's dynamic loader.
//!
//! The native build ships its modules as plain shared objects in the game's `bin/`
//! directory, already loaded by the engine itself by the time our code runs. So the
//! job here is to *find* them among the loaded objects rather than to load anything.
//!
//! The module file names in [`Module`] follow the usual Source-on-Linux convention
//! and still need to be confirmed against an actual native install.

use std::ffi::{c_char, c_int, c_void, CStr, CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use super::Module;

/// The `CreateInterface` export every Source module carries.
type CreateInterfaceFn =
    unsafe extern "C" fn(name: *const c_char, return_code: *mut c_int) -> *mut c_void;

/// What [`find_object`] digs out of a loaded ELF object.
struct LoadedObject {
    /// Full path the object was loaded from, needed to re-open it via `dlopen`.
    path: PathBuf,
    /// Start of the executable mapping.
    text_base: *const u8,
    /// Length of the executable mapping.
    text_size: usize,
}

/// Iteration state handed to the `dl_iterate_phdr` callback.
struct Search {
    /// Bare file name we are looking for, e.g. `engine.so`.
    wanted: &'static [u8],
    found: Option<LoadedObject>,
}

/// Callback for `dl_iterate_phdr`; returning non-zero stops the iteration.
unsafe extern "C" fn visit_object(
    info: *mut libc::dl_phdr_info,
    _size: usize,
    data: *mut c_void,
) -> c_int {
    let search = unsafe { &mut *(data as *mut Search) };
    let info = unsafe { &*info };

    if info.dlpi_name.is_null() {
        return 0;
    }
    let name = unsafe { CStr::from_ptr(info.dlpi_name) }.to_bytes();
    // The main executable reports an empty name; it never hosts an interface.
    if name.is_empty() {
        return 0;
    }

    let path = Path::new(OsStr::from_bytes(name));
    if path.file_name().map(OsStr::as_bytes) != Some(search.wanted) {
        return 0;
    }

    // Only the executable segment is reported. ld.so maps the alignment gaps
    // between an object's PT_LOAD segments as PROT_NONE, so spanning the whole
    // object and scanning it blindly would fault. Code lives in PF_X anyway.
    let headers = unsafe { std::slice::from_raw_parts(info.dlpi_phdr, info.dlpi_phnum as usize) };
    let Some(text) = headers
        .iter()
        .find(|h| h.p_type == libc::PT_LOAD && (h.p_flags & libc::PF_X) != 0)
    else {
        return 0;
    };

    search.found = Some(LoadedObject {
        path: path.to_path_buf(),
        text_base: (info.dlpi_addr as usize + text.p_vaddr as usize) as *const u8,
        text_size: text.p_memsz as usize,
    });
    1
}

/// Locates a loaded shared object by its bare file name.
fn find_object_named(name: &'static [u8]) -> Option<LoadedObject> {
    let mut search = Search { wanted: name, found: None };

    unsafe {
        libc::dl_iterate_phdr(Some(visit_object), &mut search as *mut Search as *mut c_void);
    }

    search.found
}

/// Locates a loaded engine module.
fn find_object(module: Module) -> Option<LoadedObject> {
    find_object_named(module.file_name().to_bytes())
}

pub(super) fn module_range(module: Module) -> Option<(*const u8, usize)> {
    find_object(module).map(|obj| (obj.text_base, obj.text_size))
}

pub(super) unsafe fn find_interface<T>(module: Module, interface_name: &CStr) -> *mut T {
    let Some(object) = find_object(module) else {
        log::warn!("Module is not loaded: {:?}", module);
        return std::ptr::null_mut();
    };

    let Ok(path) = CString::new(object.path.as_os_str().as_bytes()) else {
        return std::ptr::null_mut();
    };

    // RTLD_NOLOAD keeps this a lookup: if the engine has not loaded the module,
    // we get null instead of quietly loading a second copy of it.
    let handle = unsafe { libc::dlopen(path.as_ptr(), libc::RTLD_NOW | libc::RTLD_NOLOAD) };
    if handle.is_null() {
        log::error!("Failed to open an already-loaded {:?}", module);
        return std::ptr::null_mut();
    }

    let factory = unsafe { libc::dlsym(handle, c"CreateInterface".as_ptr()) };
    // Drops the reference `dlopen` just took. The symbol address stays valid:
    // the engine holds its own reference to the module.
    unsafe { libc::dlclose(handle) };

    if factory.is_null() {
        log::error!("'CreateInterface' not found in {:?}", module);
        return std::ptr::null_mut();
    }

    let factory: CreateInterfaceFn = unsafe { std::mem::transmute(factory) };
    unsafe { factory(interface_name.as_ptr(), std::ptr::null_mut()) as *mut T }
}

pub(super) fn module_dir_of_self() -> Option<PathBuf> {
    let mut info: libc::Dl_info = unsafe { std::mem::zeroed() };

    // Resolving by the address of this very function is what makes it "self".
    if unsafe { libc::dladdr(module_dir_of_self as *const c_void, &mut info) } == 0 {
        return None;
    }
    if info.dli_fname.is_null() {
        return None;
    }

    let path = unsafe { CStr::from_ptr(info.dli_fname) };
    Path::new(OsStr::from_bytes(path.to_bytes()))
        .parent()
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises the phdr walk against an object every glibc process has loaded.
    /// The engine modules are found exactly the same way.
    #[test]
    fn finds_a_loaded_object() {
        let libc = find_object_named(b"libc.so.6").expect("libc.so.6 is always mapped");

        assert!(!libc.text_base.is_null());
        assert!(libc.text_size > 0);
        assert!(libc.path.is_absolute(), "got {:?}", libc.path);
        assert_eq!(libc.path.file_name().unwrap(), "libc.so.6");
    }

    #[test]
    fn missing_object_is_none() {
        assert!(find_object_named(b"definitely-not-loaded.so").is_none());
    }

    /// The game's modules are not loaded into a test binary, so this is also the
    /// path taken when the SDK runs before the engine has loaded `server.so`.
    #[test]
    fn engine_modules_are_absent_outside_the_game() {
        assert!(module_range(Module::Engine).is_none());
    }

    #[test]
    fn resolves_own_directory() {
        let dir = module_dir_of_self().expect("the test binary has a path");
        assert!(dir.is_dir(), "{:?} is not a directory", dir);
    }
}
