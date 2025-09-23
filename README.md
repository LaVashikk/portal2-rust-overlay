# Portal 2 Rust Overlay

A work-in-progress project to create a D3D9-based in-game overlay for Portal 2 using Rust. It leverages `egui` for the user interface and operates by proxying the `d3d9.dll` library.

While currently tailored for Portal 2, the modular architecture is designed with potential support for other Source Engine games in mind.

## ✨ Current Status: Stable & Extensible

The project has moved beyond the "raw concept" stage and now stands on a solid, modular foundation. Thanks to a major architectural refactoring, the codebase is now split into logical crates, features a clean dependency injection model, and is ready for new features and community contributions.

The use of `egui` for the entire front-end means that **any existing `egui` application or UI component can be easily ported** into this overlay with minimal changes, making it a powerful platform for creating complex in-game tools.

### Demo

#### 1. Real-Time Game Variable Manipulation (`fogui`)

This demonstrates direct, real-time control over the Source Engine's CVar system. The `fogui` menu allows you to manipulate all aspects of the in-game fog using standard `egui` widgets like sliders, checkboxes, and a **full-featured color picker**. Changes are reflected in the game instantly, showcasing the seamless bridge between the UI and the game engine.

https://github.com/user-attachments/assets/d99e9ac5-a6ff-471c-8b4e-cc9f0139e185

#### 2. Proof-of-Concept Custom Game Menu

This is a proof-of-concept for building a completely custom, modern main menu using `egui`. It shows the potential of the overlay to not just add supplementary widgets, but to serve as a complete replacement for the game's default UI.

https://github.com/user-attachments/assets/bf2acc21-aca0-4191-a110-228df20afbf8

#### 3. Seamless Portability of `egui` Applications

One of the key design goals was to make porting existing `egui` code effortless. To prove this, the external project **[Gemini-eGUI](https://github.com/LaVashikk/Gemini-eGUI)** was integrated into the overlay. (no way, vibe modding? :D)

<img width="1280" height="719" alt="image" src="https://github.com/user-attachments/assets/2a3a405e-65b4-44c0-97e5-1e355b1a5184" />

## Features

*   **D3D9 Proxying:** Seamlessly integrates into the game's rendering pipeline.
*   **Immediate Mode GUI:** Uses the powerful and easy-to-use `egui` framework for all UI. Any existing `egui` app can be easily ported.
*   **Direct Game Engine Interaction:** Interfaces directly with Source Engine components like the CVar system to read and write game variables in real-time.

---

## Installation (For Users)
todo, later.
---

## Quick Development Start: Adding a New Window

### 1. Create Your Window Logic

All UI logic lives in the `crates/overlay-ui` crate.

Create a new file for your window, for example, `crates/overlay-ui/src/my_window.rs`. Inside, define a struct and implement the `Window` trait for it.

```rust
// in crates/overlay-ui/src/my_window.rs

use crate::{Window, SharedState};
use engine_api::Engine;

#[derive(Debug)]
pub struct MyCoolWindow {
    is_open: bool,
    counter: i32,
}
impl Default for MyCoolWindow {
    fn default() -> Self {
        Self { is_open: true, counter: 0 }
    }
}

impl Window for MyCoolWindow {
    fn name(&self) -> &'static str { "My Cool Window" }

    fn is_open(&self) -> bool { self.is_open }

    fn toggle(&mut self) { self.is_open = !self.is_open; }

    fn draw(&mut self, ctx: &egui::Context, shared_state: &mut SharedState, engine: &Engine) {
        if !shared_state.is_overlay_focused { return; } // optional feature

        egui::Window::new(self.name())
            .open(&mut self.is_open)
            .show(ctx, |ui| {
                ui.label("This is my cool new window!");
                if ui.button("Click me!").clicked() {
                    self.counter += 1;
                }
                ui.label(format!("Counter: {}", self.counter));
            });
    }
}
```

### 2. Register Your New Window

Now, tell the application about your new window. Open `crates/overlay-ui/src/lib.rs` and add your module and window instance to the `regist_windows` function.

```rust
// in crates/overlay-ui/src/lib.rs

mod debug_win;
mod fogui;
mod my_window; // 1. Declare your new module

pub fn regist_windows() -> Vec<Box<dyn Window + Send>> {
    vec![
        Box::new(OverlayText::default()),
        Box::new(debug_win::DebugWindow::default()),
        Box::new(fogui::FogWindow::default()),
        Box::new(my_window::MyCoolWindow::default()), // 2. Add an instance here
    ]
}
```

### 3. Handle Custom Input (Optional)

The `Window` trait provides an optional `on_raw_input` method. This allows your window to react to keyboard or mouse events even when the main overlay is not focused. This is perfect for implementing window-specific hotkeys.

```rust
// in your `impl Window for MyCoolWindow` block

fn on_raw_input(&mut self, umsg: u32, wparam: u16) -> bool {
    // If the 'K' key is released, toggle this window
    if umsg == windows::Win32::UI::WindowsAndMessaging::WM_KEYUP {
        if wparam == windows::Win32::UI::Input::KeyboardAndMouse::VK_K.0 {
            self.toggle();
        }
    }
    true // Return true to allow the game to also process this input
}
```

### 4. Compile the Project

Compile the project to produce the final `d3d9.dll`.

```bash
cargo build --release --target i686-pc-windows-gnu
```

The resulting DLL will be located in `target/i686-pc-windows-gnu/release/`. That's it! Your new window is now part of the overlay.

## Contributing

Contributions are welcome! Feel free to open an issue to discuss a new feature or submit a pull request with your improvements.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
