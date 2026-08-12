/// Searches for a byte pattern in a memory slice using a mask.
/// `?` in the mask means "any byte".
/// `x` in the mask means "the byte must match".
///
/// Returns `None` if `pattern` and `mask` disagree in length, so a typo in a
/// signature surfaces as "not found" rather than a bogus match.
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

#[cfg(test)]
mod tests {
    use super::find_pattern;

    #[test]
    fn matches_exact_pattern() {
        let memory = b"\x00\x11\x55\x8B\xEC\x22";
        assert_eq!(find_pattern(memory, b"\x55\x8B\xEC", "xxx"), Some(2));
    }

    #[test]
    fn wildcards_skip_bytes() {
        let memory = b"\xE8\x3B\x4D\x02\x00\xD9";
        assert_eq!(find_pattern(memory, b"\xE8\x00\x00\x00\x00\xD9", "x????x"), Some(0));
    }

    #[test]
    fn reports_first_match_only() {
        let memory = b"\x90\x90\x90";
        assert_eq!(find_pattern(memory, b"\x90", "x"), Some(0));
    }

    #[test]
    fn rejects_mask_length_mismatch() {
        let memory = b"\x55\x8B\xEC";
        assert_eq!(find_pattern(memory, b"\x55\x8B\xEC", "xx"), None);
    }

    #[test]
    fn no_match_returns_none() {
        assert_eq!(find_pattern(b"\x01\x02", b"\x03\x04", "xx"), None);
    }

    /// A pattern longer than the haystack must not panic - `windows()` yields
    /// nothing in that case.
    #[test]
    fn pattern_longer_than_memory() {
        assert_eq!(find_pattern(b"\x55", b"\x55\x8B", "xx"), None);
    }
}
