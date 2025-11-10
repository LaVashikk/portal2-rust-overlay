use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::ProcessStatus::{GetModuleInformation, MODULEINFO};
use windows::Win32::System::Threading::GetCurrentProcess;
use windows::core::PCSTR;

/// Returns the base address and size of a module in memory.
/// `module_name` must be a null-terminated string, e.g., b"engine.dll\0".
pub unsafe fn get_module_memory_range(module_name: &'static [u8]) -> Option<(*const u8, usize)> {
    // 1. Get the module handle
    let module_handle = match unsafe { GetModuleHandleA(PCSTR(module_name.as_ptr())) } {
        Ok(handle) if !handle.is_invalid() => handle,
        _ => return None,
    };

    // 2. Get module information (base address, size)
    let mut module_info = MODULEINFO::default();
    let process_handle = unsafe { GetCurrentProcess() };

    if unsafe { GetModuleInformation(
        process_handle,
        module_handle,
        &mut module_info,
        std::mem::size_of::<MODULEINFO>() as u32,
    ).is_ok() } {
        let base = module_info.lpBaseOfDll as *const u8;
        let size = module_info.SizeOfImage as usize;
        Some((base, size))
    } else {
        None
    }
}

/// Searches for a byte pattern in a memory slice using a mask.
/// `?` in the mask means "any byte".
/// `x` in the mask means "the byte must match".
pub fn find_pattern(memory: &[u8], pattern: &[u8], mask: &str) -> Option<usize> {
    // The length of the pattern and mask must be the same
    if pattern.len() != mask.len() {
        return None;
    }

    let pattern_len = pattern.len();

    // Iterate over "windows" in memory, the size of each window is equal to the length of the pattern
    for (i, window) in memory.windows(pattern_len).enumerate() {
        // Check if the current window matches the pattern
        let is_match = window.iter()
            .zip(pattern.iter())
            .zip(mask.chars())
            .all(|((&mem_byte, &pat_byte), mask_char)| {
                // If the mask is 'x', the bytes must match. If '?', skip the check.
                mask_char == '?' || mem_byte == pat_byte
            });

        if is_match {
            // If a match is found, return the offset from the beginning of the memory slice
            return Some(i);
        }
    }

    // If nothing is found
    None
}
