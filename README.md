<div align="center">
<img src="https://github.com/user-attachments/assets/37220616-a2f5-47be-9811-03d6a3aceffa" alt="Logo" >

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20x86-blue)](https://www.microsoft.com/windows)
[![Source Engine](https://img.shields.io/badge/Source%20Engine-Compatible-orange)](https://developer.valvesoftware.com/wiki/Source)

This repository is a powerful **in-game modding plugin** and a public template for creating custom tools and UIs for Portal 2, built with Rust and the `egui` library. It comes "batteries-included" with a suite of professional-grade tools for modders and developers alike.

**[Get Started](#-quick-install--test) • [Built-in Tools](#-built-in-modding-tools) • **[Showcase](#-showcase)** • [Documentation](CONTRIBUTING.md)** • **[Support](https://github.com/LaVashikk/portal2-rust-overlay/issues)**
</div>

---

## 🚀 Quick Install & Test

The easiest way to try the overlay is with a pre-built version.

1. Go to the [**Releases Page**](https://github.com/LaVashikk/portal2-rust-overlay/releases) and download the `injector_server_plugin.zip`.
2. Extract the contents into your `...Portal 2/portal2/` folder.
3. Launch the game.
4.  Press **F3** in-game to toggle the overlay menu's focus.

> [!NOTE]
> This overlay does **not** support Portal 2: Community Edition (P2:CE). For a detailed explanation, please see the [P2:CE Support Notice](P2CE_SUPPORT.md).

### What You Can Build

Use this framework as a foundation for a wide variety of tools:
- **Debug Tools** - Real-time variable monitoring, performance profilers
- **Gameplay Enhancements** - Custom HUDs, information overlays
- **Development Tools** - Entity inspectors, playtest surveys
- **Anything** - no, i'm serious, you can do anything!


# 🛠 Built-in Modding Tools

Version 1.0.0 transforms this framework into a ready-to-use modding toolkit. The following tools are available out-of-the-box:

<img width="1913" height="1082" alt="2026-05-04_22-47" src="https://github.com/user-attachments/assets/b5505780-4225-4984-b9d6-41ebe9dee171" />

### Material Inspector
Edit `.vmt` files in real-time! Point your crosshair at any surface, grab its material, change properties, validate texture paths, and preview the results instantly without restarting the map.

### Better Fog GUI
Easily manipulate fog parameters on the fly. Sync settings directly from `env_fog_controller` entities on the map to find the perfect atmospheric look.

### Post-Processing & Color Correction
A dedicated menu for tweaking Bloom, Autoexposure, Motion Blur, and Color Correction LUTs (`.raw` files). Test color grading instantly inside the engine.

### Engine Debug Menu
Quick toggles for engine performance, Material System flags (`mat_wireframe`, `mat_fullbright`), Renderer debug modes, and crosshair entity inspection.

---

# ✨ Showcase

## Projects Built with this Framework

- [Playtest Tool](https://github.com/LaVashikk/portal2-playtest-tool) — advanced in-game feedback and bug reporting
- [VMF to PBR](https://github.com/LaVashikk/VMF-to-PBR) — in-game interface for editing PBR lightning on Vanilla Portal 2
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

--- 

# 🧑‍💻 For Developers

Ready to create your own tools? This project is a template designed for extension. All development instructions, from setting up your environment to building from source and adding new windows, are in our comprehensive **[Developer Guide](CONTRIBUTING.md)**.

## Project Structure

```
crates/
├── injector_*/          # Entry points (don't modify)
├── hook_core/           # D3D9 hooking core
├── overlay_types/       # Shared types, event system, and hotkey abstractions
├── overlay_runtime/     # Manages UI state, input, and rendering loop
├── egui_backend/        # The egui rendering backend for D3D9
├── portal2_sdk/         # Safe bindings to Source Engine functions
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
| **Performance drops** | Reduce UI complexity |

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
