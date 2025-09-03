# Portal 2 Rust Overlay

A work-in-progress project to create a D3D9-based in-game overlay for Portal 2 using Rust. It leverages `egui` for the user interface and operates by proxying the `d3d9.dll` library.

While currently tailored for Portal 2, the architecture is designed with potential support for other Source Engine games in mind.

## ⚠️ Current Status: Raw Concept

This project is in a highly experimental state. It serves as a proof-of-concept for building complex game overlays in Rust.

 > [!NOTE]
 > It is **not possible to compile this repository as-is** due to a dependency on a locally modified version of the `egui-d3d9` crate. This will be resolved in the future by migrating the project to a Cargo workspace and including the modified dependency directly.

### Demo

Here is a short video demonstrating the current functionality:

https://github.com/user-attachments/assets/d99e9ac5-a6ff-471c-8b4e-cc9f0139e185

## Quick Start: Adding a New Window

If you have the local dependencies set up, here is how to add a new UI window to the overlay:

1.  **Create your window logic.**
    Create a new module in `src/overlay/` (e.g., `my_window.rs`). Your window struct must implement the `Window` trait defined in `src/overlay/utils.rs`.

2.  **Register your new window.**
    Add an instance of your window to the `windows` vector inside the `UiManager::new()` function in `src/overlay/mod.rs`.

    ```rust
    // in src/overlay/mod.rs
    mod my_window; // Don't forget to declare the module

    impl UiManager {
        pub fn new() -> Self {
            Self {
                windows: vec![
                    // ... existing windows
                    Box::new(fogui::FogWindow::default()),
                    // Add your new window here
                    Box::new(my_window::MyCoolWindow::default()),
                ],
                // ...
            }
        }
    }
    ```

3.  **Compile for the correct target.**
