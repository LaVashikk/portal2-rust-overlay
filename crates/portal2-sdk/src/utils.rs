//! Miscellaneous helpers that don't belong to any particular engine interface.

/// Returns the directory the plugin binary itself lives in.
///
/// Handy for loading files shipped alongside the plugin without depending on the
/// game's working directory, which Source changes on its own.
///
/// Despite the name, this works on the native Linux build too, where the binary is
/// a `.so` rather than a `.dll`.
pub fn get_dll_directory() -> Option<std::path::PathBuf> {
    crate::platform::module_dir_of_self()
}
