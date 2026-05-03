# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0-testing.3] - 2026-05-03

### Added
- **Global Event System**: `OverlayEvent` enum and global event bus (`events::push_event`) for cross-thread and cross-window communication.
- **Input & Hotkey System**: `KeyCode` mappings and `HotkeyManager` for assigning shortcuts to framework events.
- **`overlay_types` crate**: Contains shared abstractions (Events, Input, etc.).
- **Top Panel Window**: A default control bar to easily toggle registered windows and manage overlay features.
- `SharedState` now acts as a centralized "Single Source of Truth", allowing modders to easily add custom global fields.

### Changed
- **Major Window Trait Refactor**: 
  - Replaced `toggle()` with `set_open(open: bool)`.
  - Replaced low-level `on_raw_input()` with a high-level `on_event(&mut self, event: &OverlayEvent, shared_state: &mut SharedState)` method.
- Moved window registration logic to `crates/custom_windows/src/custom.rs` (`regist()` handles hotkeys, game events, and windows).
- Moved user custom windows into the `crates/custom_windows/src/custom/` directory.


## [0.1.0-testing.2] - 2025-11-18

### Added
- Reset method for ConVar
- Focus captured overlay message (#5)

### Changed
- Improved logging and error handling
- Renamed cursor state variable for clarity

## [0.1.0-testing.1] - 2025-11-08

### Added
- Initial pre-release for testing
