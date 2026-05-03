# Contributing & Development Guide

First off, thank you for considering contributing! We welcome any improvements, whether it's fixing a bug, adding a new feature to the template, or simply making a suggestion. This guide is designed to get you started quickly.

## Table of Contents

- [Prerequisites & Setup](#-prerequisites--setup)
- [Project Structure](#project-structure)
- [Development Workflow](#-development-workflow)
  - [Adding a New Window](#adding-a-new-window)
  - [Events, Hotkeys & Global State](#events-hotkeys--global-state)
  - [Advanced Examples](#advanced-examples)
- [Building & Testing](#-building--testing)
- [Keeping Your Fork Up-to-Date](#-keeping-your-fork-up-to-date)
- [Adding Your Project to the Showcase](#-adding-your-project-to-the-showcase)
- [Code & Commit Guidelines](#-code--commit-guidelines)
- [FAQ](#frequently-asked-questions)

## Prerequisites & Setup

Before you begin, make sure you have the following installed:
-   **Rust Toolchain**: Including `rustup`, `cargo`, and `rustc`.
-   **32-bit Windows Target**: This framework builds 32-bit DLLs for Source Engine games.
-   **Git**: For version control.


```bash
# Install Rust from https://rustup.rs/ if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the 32-bit MSVC target required for building
rustup target add i686-pc-windows-msvc

# Clone the repository
git clone https://github.com/LaVashikk/portal2-rust-overlay
cd portal2-rust-overlay

# Build
cargo build --release --target i686-pc-windows-msvc
```

## Project Structure
The project is organized into a multi-crate workspace. The most important directory for you is `custom_windows`.


```
portal2-rust-overlay/
├── crates/
│   ├── injector_d3d9_proxy/      # D3D9 proxy entry point
│   ├── injector_server_plugin/   # Server plugin entry point  
│   ├── injector_client_wrapper/  # Client.dll wrapper
│   ├── hook_core/                # Core D3D9 hooking logic
│   ├── overlay_types/            # Shared types, events, and input abstractions
│   ├── overlay_runtime/          # UI management & rendering
│   ├── egui_backend/             # Egui-D3D9 integration
│   ├── portal2_sdk/              # Source Engine FFI bindings
│   └── custom_windows/           # ← YOUR UI CODE GOES HERE
├── docs/                         # Additional documentation
├── Cargo.toml                    # Workspace configuration
└── README.md
```

-   **`custom_windows`**: This is your primary workspace. Add all your UI windows here.
-   **`portal2_sdk`**: Contains safe bindings to Source Engine functions. Feel free to extend this to access more engine features.

## Development Workflow

### Adding a New Window

#### Step 1: Create the Window File

Create a new file in the custom directory, for example `crates/custom_windows/src/custom/my_window.rs`:

```rust
use crate::{Window, SharedState};
use overlay_types::events::OverlayEvent;
use portal2_sdk::Engine;
use egui::Context;

#[derive(Debug, Default)]
pub struct MyWindow {
    is_open: bool,
    // Add your state here
    counter: i32,
    text_buffer: String,
}

impl Window for MyWindow {
    fn name(&self) -> &'static str { "My Window" }
    fn set_open(&mut self, open: bool) { self.is_open = open; }
    fn is_open(&self) -> bool { self.is_open }

    // Optional: React to global overlay events
    fn on_event(&mut self, event: &OverlayEvent, _shared_state: &mut SharedState) {
        if let OverlayEvent::Command(cmd) = event {
            if cmd == "my_custom_action" {
                self.counter += 1;
            }
        }
    }

    // Optional: Only draw when overlay is focused
    fn is_should_render(&self, shared_state: &SharedState, _engine: &portal2_sdk::Engine) -> bool {
        shared_state.is_overlay_focused
    }

    fn draw(&mut self, ctx: &Context, shared: &mut SharedState, engine: &Engine) {
        egui::Window::new(self.name())
            .open(&mut self.is_open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("My Custom Window");
                
                // Example: Button with game action
                if ui.button("Click Me").clicked() {
                    self.counter += 1;
                    // Execute game command
                    engine.client().execute_client_cmd_unrestricted(
                        &format!("echo Button clicked {} times", self.counter)
                    );
                }
                
                // Example: Text input to run console commands
                ui.horizontal(|ui| {
                    ui.label("Command:");
                    ui.text_edit_singleline(&mut self.text_buffer);
                    if ui.button("Execute").clicked() {
                        engine.client().execute_client_cmd_unrestricted(&self.text_buffer);
                    }
                });
                
                // Example: Read CVar
                if let Some(fps_max) = engine.cvar_system().find_var("fps_max") {
                    ui.label(format!("FPS Limit: {}", fps_max.get_int()));
                }
                // Example: Triggering the global event bus!
                if ui.button("Trigger Custom Event").clicked() {
                    overlay_types::events::push_event(OverlayEvent::Command("my_custom_action".to_string()));
                }
            });
    }
}
```

#### Step 2: Register the Window & Hotkeys

Open `crates/custom_windows/src/custom.rs`. This file is the central registration hub for all UI components, engine events, and hotkeys.

```rust
mod my_window; // 1. Add module declaration

pub fn regist_windows() -> Vec<Box<dyn Window + Send>> {
    vec![
        // ... existing windows
        Box::new(my_window::MyWindow::default()), // 2. Add your window instance here
    ]
}

fn regist_hotkeys(_engine: &Engine, hotkeys_manager: &mut HotkeyManager) {
    // ... existing hotkeys

    // 3. Bind a key to toggle your new window!
    // The `false` means this input is consumed and won't be passed to the game.
    hotkeys_manager.bind(KeyCode::F5, OverlayEvent::ToggleWindow("My Window"), false);
}
```

#### Step 3: Build & Test

Use the convenient cargo aliases defined in `.cargo/config.toml` to build a specific injector.

```bash
# Build the D3D9 Proxy (most common)
cargo build-d3d9

# Build the Server Plugin (for Portal 2)
cargo build-plugin

# Build the Client Wrapper (advanced)
cargo build-client
```
The output DLL will be in `target/i686-pc-windows-msvc/release/`.

### Events, Hotkeys & Global State

The framework provides powerful cross-communication abstractions via the `overlay_types` crate.

**Global Shared State**
You can add your own custom mod state to `SharedState` in `crates/custom_windows/src/lib.rs`. Since this struct is passed mutably to all window `draw` and `on_event` calls, it acts as a perfect single source of truth for your data!

**The Event Bus**
To decouple windows and logic, you can send events asynchronously via `events::push_event(OverlayEvent)`. 
* Built-in events include: `ToggleOverlay`, `ToggleWindow("Name")`, `SetWindowState("Name", bool)`, `EngineCommand`, etc.
* Custom events can be dispatched via `OverlayEvent::Command(String)`.

**Hotkeys**
Instead of manually intercepting low-level WinAPI messages, you can map high-level keys to `OverlayEvent`s inside the `regist_hotkeys` function in `custom.rs`. The framework automatically figures out if it should intercept the key press or pass it down to the Source engine.

### Advanced Examples

<details>
<summary><b>Working with CVars</b></summary>

```rust
// Read CVar
if let Some(cvar) = engine.cvar_system().find_var("sv_gravity") {
    let gravity = cvar.get_float();
    ui.label(format!("Gravity: {}", gravity));
}

// Write CVar (requires sv_cheats for protected cvars)
if let Some(mut cvar) = engine.cvar_system().find_var("host_timescale") {
    let mut timescale = cvar.get_float();
    if ui.add(egui::Slider::new(&mut timescale, 0.1..=10.0)).changed() {
        cvar.set_value_float(timescale);
    }
}

// Execute console commands
engine.client().execute_client_cmd_unrestricted("sv_cheats 1");
```
</details>

<details>
<summary><b>Player Information</b></summary>

```rust
// Get the local player's info
let local_idx = engine.client().get_local_player();
if let Some(info) = engine.client().get_player_info(local_idx) {
    ui.label(format!("Name: {}", info.name()));
    ui.label(format!("SteamID: {}", info.guid()));
}

// Iterate over all possible player slots
ui.separator();
ui.heading("All Players:");
for i in 1..=engine.client().get_max_clients() {
    if let Some(info) = engine.client().get_player_info(i) {
        if !info.name().is_empty() { // Only show connected players
            ui.label(format!("- Player {}: {}", i, info.name()));
        }
    }
}
```
</details>

<details>
<summary><b>Custom Drawing</b></summary>

```rust
// Draw outside of windows
ctx.debug_painter().text(
    egui::pos2(10.0, 10.0),
    egui::Align2::LEFT_TOP,
    "Overlay Active",
    egui::FontId::proportional(20.0),
    egui::Color32::from_rgb(255, 128, 0),
);

// Draw shapes
ctx.debug_painter().circle_filled(
    egui::pos2(100.0, 100.0),
    50.0,
    egui::Color32::from_rgba_unmultiplied(255, 0, 0, 128),
);
```
</details>

## Keeping Your Project Up-to-Date

If you created your project using the "Use this template" button, your repository starts as a separate entity. To pull in bug fixes and new features from this original template, you need to configure it as a remote `upstream`.

This is a one-time setup. In your project's local repository, run:
```bash
# Add the original template repository as a remote named "upstream"
git remote add upstream https://github.com/LaVashikk/portal2-rust-overlay

# Fetch the upstream history and merge it. The --allow-unrelated-histories flag is
# necessary for the first pull because your project and the template don't share a common Git history.
git pull upstream main --allow-unrelated-histories
```
After this initial setup, you can keep your project updated by simply running:
```Bash
git pull upstream main
```
## 🌟 Adding Your Project to the Showcase

If you've built something cool, we'd love to feature it!

1.  **Add the `p2-rust-overlay-project` topic** to your GitHub repository.
2.  Fork this repository and edit the `README.md` file.
3.  Add a bullet point for your project in the "Projects Built with this Framework" section using this format:
    ```markdown
    - [**Your Project Name**](https://github.com/your-username/your-repo) — A brief, one-line description of your tool.
    ```
4.  Submit a pull request!


## Code & Commit Guidelines

### Style

-   **Style**: Run `cargo fmt` before committing. Use `cargo clippy --all-targets` to catch common issues.
-   **Commits**: Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification (e.g., `feat:`, `fix:`, `docs:`). This helps keep the history clean.
-   **Unsafe Code**: Keep `unsafe` blocks as small as possible and add a `// SAFETY:` comment explaining why the code is safe.


## Frequently Asked Questions

<details>
<summary><b>Q: How do I debug my window?</b></summary>

Use the `log` crate, which is already set up. Logs are written to `d3d9_proxy_mod.log` in the game's root directory.

```rust
log::info!("Window opened");
log::error!("Failed to find cvar: {}", name);
```
</details>

<details>
<summary><b>Q: Can I use external crates?</b></summary>

Yes! Add dependencies to `crates/custom_windows/Cargo.toml`:
```toml
[dependencies]
serde = "1.0"
reqwest = "0.11"
```
</details>

<details>
<summary><b>Q: How do I persist window state?</b></summary>

Implement save/load in your window:
```rust
impl MyWindow {
    fn save_config(&self) -> Result<(), Box<dyn Error>> {
        let json = serde_json::to_string(self)?;
        std::fs::write("my_window.json", json)?;
        Ok(())
    }
    
    fn load_config(&mut self) -> Result<(), Box<dyn Error>> {
        let json = std::fs::read_to_string("my_window.json")?;
        *self = serde_json::from_str(&json)?;
        Ok(())
    }
}
```
</details>

## Getting Help

- 📋 [Open an Issue](https://github.com/LaVashikk/portal2-rust-overlay/issues/new)
- 💬 [Start a Discussion](https://github.com/LaVashikk/portal2-rust-overlay/discussions)
- 📚 [Read egui docs](https://docs.rs/egui)
- 🎮 [Source SDK Wiki](https://developer.valvesoftware.com/wiki/portal2_sdk)

---

Thank you for contributing! 🎮🦀
