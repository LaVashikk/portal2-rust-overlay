
# Portal 2 Rust Overlay Framework

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20x86-blue)](https://www.microsoft.com/windows)
[![Source Engine](https://img.shields.io/badge/Source%20Engine-Compatible-orange)](https://developer.valvesoftware.com/wiki/Source)

This repository is a powerful framework and public template for creating custom in-game tools and UIs for Source Engine games, built with Rust and the `egui` library. Click the **"Use this template"** button above to get started with a clean copy of the framework for your own project.

**[Get Started](#-quick-start)** • **[Features](#-features)** • **[Showcase](#-showcase)** • **[Documentation](CONTRIBUTING.md)** • **[Support](https://github.com/LaVashikk/portal2-rust-overlay/issues)**

---

# 🚀 Quick Test (Using Pre-built Release)

The easiest way to try the overlay is with a pre-built version.

1.  Go to the [**Releases Page**](https://github.com/LaVashikk/portal2-rust-overlay/releases) and download the `injector_d3d9_proxy.zip` file.
2.  Extract `d3d9.dll` from the zip file.
3.  Place `d3d9.dll` into your game's `bin` folder (e.g., `C:\Steam\steamapps\common\Portal 2\bin`).
4.  Launch the game.
5.  Press **F3** in-game to toggle the overlay menu's focus.

### What You Can Build

Use this framework as a foundation for a wide variety of tools:
- **Debug Tools** - Real-time variable monitoring, performance profilers
- **Gameplay Enhancements** - Custom HUDs, information overlays
- **Development Tools** - Entity inspectors, playtest surveys
- **Training Tools** - Practice modes, trajectory visualizers

# ✨ Showcase

## Projects Built with this Framework

- [Portal 2 Survey Tool](https://github.com/LaVashikk/portal2-survey-tool) — advanced in-game feedback and bug reporting
- Your project here — see how to add it in [CONTRIBUTING.md](CONTRIBUTING.md#adding-your-project-to-showcase)

> [TIP] 
> Add the `p2-rust-overlay-project` topic to your repo for discoverability!


## Demonstrations

### Real-Time Engine Control
Direct manipulation of game variables with immediate visual feedback:

https://github.com/user-attachments/assets/d99e9ac5-a6ff-471c-8b4e-cc9f0139e185

### Custom UI Replacement
Modern, responsive interface replacing game's default UI:

https://github.com/user-attachments/assets/bf2acc21-aca0-4191-a110-228df20afbf8

### External App Integration
Any `egui` application can be ported seamlessly:

<img width="1280" alt="Gemini-eGUI integration" src="https://github.com/user-attachments/assets/2a3a405e-65b4-44c0-97e5-1e355b1a5184" />

## Installation Guide

## Prerequisites

- Windows 10/11 (x64 with x86 game support)
- [Rust toolchain](https://rustup.rs/) with `i686-pc-windows-msvc` or `i686-pc-windows-gnu` target
- Visual Studio 2019+ with C++ tools OR MinGW-w64
- Source Engine game

## Choose Your Injection Method
This section explains how to install pre-built releases for each injection method.

<table>
<tr>
<th>Method</th>
<th>Output File</th>
<th>Best For</th>
<th>Pros</th>
<th>Cons</th>
</tr>
<tr>
<td><b>D3D9 Proxy</b><br><code>injector_d3d9_proxy</code></td>
<td><code>d3d9.dll</code></td>
<td>Most Source games</td>
<td>✓ Universal<br>✓ Simple setup<br>✓ No game files modified</td>
<td>⚠️ No Vulkan Support<br>⚠️ Installation in sourcemods is not possible</td>
</tr>
<tr>
<td><b>Server Plugin</b><br><code>injector_server_plugin</code></td>
<td><code>plugin.dll</code></td>
<td><b>Portal 2 (recommended)</b></td>
<td>✓ Clean integration<br>✓ Works with sourcemods<br>✓ Easy to remove<br>✓ Hot-swap in runtime<br>✓ Vulkan Support</td>
<td>⚠️ Plugin support required. Tested only with Portal 2</td>
</tr>
<tr>
<td><b>Client Wrapper</b><br><code>injector_client_wrapper</code></td>
<td><code>client.dll</code></td>
<td>Advanced scenarios</td>
<td>✓ Deep integration<br>✓ Vulkan Support</td>
<td>⚠️ Modifies game files<br>⚠️ Complex setup</td>
</tr>
</table>


<br>

<details>
<summary><strong>Method 1: D3D9 Proxy (Universal)</strong></summary>

1.  Download `injector_d3d9_proxy.zip` from the [Releases Page](https://github.com/LaVashikk/portal2-rust-overlay/releases).
2.  Place the extracted `d3d9.dll` into your game's `bin` directory (e.g., `C:\...\[GAME]\bin\`).
</details>

<details>
<summary><strong>Method 2: Server Plugin (Portal 2 Recommended)</strong></summary>

1.  Download `injector_server_plugin.zip` from the [Releases Page](https://github.com/LaVashikk/portal2-rust-overlay/releases).
2.  Place the extracted `egui_overlay_plugin.dll` into `...Portal 2\portal2\addons\`.
3.  Create a new text file named `overlay.vdf` in the `addons` folder with the following content:
    ```vdf
    "Plugin"
    {
        "file"		"addons/egui_overlay_plugin"
    }
    ```
</details>

<details>
<summary><strong>Method 3: Client Wrapper (Advanced)</strong></summary>

1.  Go to your game's `bin` folder (e.g., `.../Portal 2/portal2/bin/`).
2.  **Backup your original `client.dll`** by renaming it to `client_original.dll`.
3.  Download `injector_client_wrapper.zip` from the [Releases Page](https://github.com/LaVashikk/portal2-rust-overlay/releases).
4.  Place the extracted `client.dll` into the `bin` folder.
</details>

---

# 🧑‍💻 For Developers

Ready to create your own tools? This project is a template designed for extension. All development instructions, from setting up your environment to building from source and adding new windows, are in our comprehensive **[Developer Guide](CONTRIBUTING.md)**.

## Project Structure

```
crates/
├── injector_*/          # Entry points (don't modify)
├── hook_core/           # D3D9 hooking core
├── overlay_runtime/     # Manages UI state, input, and rendering loop
├── egui_backend/        # The egui rendering backend for D3D9
├── source_sdk/          # Safe bindings to Source Engine functions
└── custom_windows/      # **YOUR CODE GOES HERE! 🎯**
```

## Troubleshooting

<details>
<summary><b>Common Issues & Solutions</b></summary>

| Issue | Solution |
|-------|----------|
| **Overlay not appearing** | Press `F3` to toggle focus. Check the in-game console and `d3d9_proxy_mod.log` (in the game directory) for errors. |
| **Game crashes on start** | Ensure you are using a 32-bit game. Verify game files in Steam. Make sure you placed the DLL in the correct folder (`bin` is common). |
| **Mouse input doesn't work** | Run the game in windowed or borderless-windowed mode. |
| **Performance drops** | Disable VSync, reduce UI complexity |

</details>

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Setting up development environment
- Keeping your fork updated
- Submitting pull requests
- Code style guidelines

## Acknowledgments

This project wouldn't have been possible without the inspiration and help from several projects and individuals in the community.

- [egui](https://github.com/emilk/egui) - Immediate mode GUI framework
- [p2-rtx](https://github.com/xoxor4d/p2-rtx) for inspiring the project and showing that creating external custom GUIs was possible.
- [Portal 2 Multiplayer Mod Plugin](https://github.com/Portal-2-Multiplayer-Mod/Portal-2-Multiplayer-Mod-Plugin) for serving as a valuable codebase and reference.

Special thanks to **[@OrsellGit](https://github.com/OrsellGit)** and **[@0xNULLderef](https://github.com/0xNULLderef)** for their invaluable technical assistance with the Source Engine plugin system.


## License

MIT License - see [LICENSE](LICENSE) for details. Use freely in your projects!

---

<p align="center">
  <b>Ready to build your own overlay?</b><br>
  <a href="https://github.com/LaVashikk/portal2-rust-overlay/generate">Use this template</a> •
  <a href="CONTRIBUTING.md">Read the docs</a> •
  <a href="https://github.com/LaVashikk/portal2-rust-overlay/issues">Get help</a>
</p>
