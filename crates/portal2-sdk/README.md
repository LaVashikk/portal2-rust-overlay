# Portal 2 Rust SDK

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20x86-blue)](https://www.microsoft.com/windows)
[![Source Engine](https://img.shields.io/badge/Source%20Engine-Compatible-orange)](https://developer.valvesoftware.com/wiki/Source)

A safe, ergonomic, and powerful Rust SDK designed specifically for Portal 2 modding and server plugin development. 

Building upon the foundation of our earlier project, [source-plugin-rs](https://github.com/LaVashikk/source-plugin-rs), this crate takes Rust plugin development to the next level by completely eliminating the dreaded "C++ header hell" and complex FFI generation. Instead of linking against massive Valve C++ SDKs, this framework uses **dynamic memory signature scanning** and interface extraction to talk to the Source Engine natively and safely from Rust.

## ⚠️ Important Requirements

* **Windows Only:** This SDK interacts directly with the Windows API (`GetProcAddress`, `GetModuleHandleA`, etc.). 
* **32-Bit (x86) Architecture:** The Source Engine (Portal 2) is a 32-bit application. You **must** compile your projects using the `i686-pc-windows-msvc` or `i686-pc-windows-gnu` Rust targets.

## Core Features

* **No C++ Bindings:** All engine interfaces are retrieved at runtime via signature scanning and vtable indexing. Zero C++ headers required.
* **Idiomatic Rust API:** We wrap raw pointers and unsafe C strings into safe, ergonomic Rust types (`Option`, `String`, standard Rust iterators).
* **Entity Management (`IServerTools`):** Iterate over entities, query KeyValues, manipulate properties, and spawn objects dynamically.
* **Game Event Listeners:** Safely hook into `IGameEventManager2` using standard Rust closures.
* **CVar System (`ICvar`):** Read, modify, and flag console variables natively.
* **Engine Interactions (`IVEngineClient` & `IVEngineServer`):** Execute console commands, query player info, manipulate view angles, check map states, and much more.

## Quick Start

Add the SDK to your `Cargo.toml`:

```toml
[dependencies]
portal2-sdk = "0.1.0"
```

Ensure your `.cargo/config.toml` is set up to build for 32-bit Windows:
```toml
[build]
target = "i686-pc-windows-msvc" # or i686-pc-windows-gnu
```

## Usage Examples

Here is a glimpse of how easy it is to interact with Portal 2 using this SDK.

### 1. Initialization
The engine must be initialized once. This step scans memory for signatures and resolves all internal pointers.

```rust
use portal2_sdk::Engine;

// Initialize the engine interfaces (Call this once, e.g., on plugin load or thread start)
let engine = match Engine::initialize() {
    Ok(inst) => inst,
    Err(e) => panic!("Failed to initialize Portal 2 SDK: {}", e),
};
```

### 2. Executing Console Commands
Easily run commands as if you typed them in the developer console.

```rust
let client = engine.client();

// Execute a client command without restrictions (bypasses sv_cheats in some contexts)
client.execute_client_cmd_unrestricted("echo Hello from Rust!");
client.execute_client_cmd_unrestricted("sv_gravity 300");
```

### 3. Printing Directly to the Developer Console (`~`)
No more hacks with `echo` commands! The SDK provides `con_print!` and `con_color_print!` macros to output text directly into the in-game developer console using native `ICvar` virtual methods.

```rust
use portal2_sdk::{con_print, con_color_print, Color};

// Standard white console text
con_print!("Hello from Rust plugin! Loaded successfully.\n");

// Colored console text (Green, Yellow, Red, etc.)
con_color_print!(Color::rgb(0, 255, 0), "[OK] Mod initialized without errors.\n");
con_color_print!(Color::rgb(255, 200, 0), "[WARN] High entity count: {}\n", 1024);
```

### 4. Registering Custom Console Commands (`ConCommand`)
You can register new custom console commands with native Rust callbacks that receive command-line arguments (`CCommand`).

```rust
use portal2_sdk::{ConCommand, CCommand, CvarFlags, con_print, con_color_print, Color};

extern "C" fn my_teleport_callback(cmd: &CCommand) {
    con_print!("Command called! Total arguments: {}\n", cmd.arg_count());

    if let Some(target) = cmd.arg(1) {
        con_color_print!(Color::rgb(0, 255, 255), "Teleporting to target: {}\n", target);
    } else {
        con_color_print!(Color::rgb(255, 50, 50), "Usage: my_teleport <destination>\n");
    }
}

// Register command inside the engine:
ConCommand::register_new(
    "my_teleport",
    "Teleports the player to a target location",
    CvarFlags::NONE,
    my_teleport_callback,
).expect("Failed to register custom ConCommand");
```

### 5. Registering Custom Console Variables (`ConVar`)
You can create new console settings (`ConVar`) with custom default values, flags, and optional min/max bounds using our high-level builder API or shortcut methods.

```rust
use portal2_sdk::{ConVar, CvarFlags};

// Using ConVarBuilder for bounded variables:
let fov_cvar = ConVar::builder("my_custom_fov", "90.0")
    .help_text("Custom field of view setting")
    .flags(CvarFlags::ARCHIVE)
    .min(60.0)
    .max(140.0)
    .register()
    .expect("Failed to register ConVar");

println!("Current custom FOV: {}", fov_cvar.get_float());

// Or using register_new shortcut:
ConVar::register_new(
    "open_survey",
    "0",
    "Target survey config path or 0 to close",
    CvarFlags::NONE,
).expect("Failed to register open_survey");
```

### 6. Manipulating Existing CVars
Find and modify built-in console variables, even overriding their protected flags.

```rust
let cvar_system = engine.cvar_system();

if let Some(mut cheats_cvar) = cvar_system.find_var("sv_cheats") {
    // Read the value
    println!("Current sv_cheats value: {}", cheats_cvar.get_int());
    
    // Modify the value natively
    cheats_cvar.set_value_int(1);
    
    // You can even remove flags (like HIDDEN or CHEAT)
    cheats_cvar.remove_flags(portal2_sdk::CvarFlags::HIDDEN);
}
```

### 4. Listening to Game Events
Hook into game events using safe Rust closures. The SDK automatically handles the dispatching.

```rust
let event_manager = engine.game_event_manager();

// Listen to player spawns
event_manager.listen("player_spawn", |event| {
    let user_id = event.get_int("userid", 0);
    println!("Player with UserID {} just spawned!", user_id);
});

// Listen to portal placements
event_manager.listen("portal_fired", |event| {
    let is_portal2 = event.get_bool("leftportal", false);
    let color = if is_portal2 { "Blue" } else { "Orange" };
    println!("Fired a {} portal!", color);
});
```

### 5. Finding and Iterating Entities
The SDK provides a neat `Entities` wrapper allowing you to use standard Rust iterators on the server's entity list.

```rust
let entities = engine.entities();

// Standard Iterator usage
for ent in entities.iter().filter(|ent| ent.get_classname() == "prop_weighted_cube") {
    println!("Found a companion cube at origin: {}", ent.get_origin(engine.server_tools()));
}

// Or use the built-in finders
if let Some(player) = entities.find_by_classname(None, "player") {
    println!("Player Health: {}", player.get_health(engine.server_tools()));
    
    // Teleport the player slightly upwards
    let mut origin = player.get_origin(engine.server_tools());
    origin.z += 50.0;
    
    engine.server_tools().snap_player_to_position(
        &origin, 
        &player.get_angles(engine.server_tools()), 
        None
    );
}
```

### 6. Querying Player Info
Access details about players currently connected to the server.

```rust
let local_player_idx = engine.client().get_local_player();

if let Some(player_info) = engine.client().get_player_info(local_player_idx) {
    println!("Name: {}", player_info.name());
    println!("SteamID (GUID): {}", player_info.guid());
    println!("Is Bot? {}", player_info.fake_player);
}
```

## 🧠 How it Works Under the Hood

Unlike traditional Source Engine plugins that rely on linking huge `.lib` files and `#include`ing hundreds of C++ headers:
1. We use Windows APIs (`GetModuleHandle`, `GetProcAddress`) to find the engine's `CreateInterface` export.
2. We grab the base pointer (`this`) for major interfaces like `VEngineClient015` or `VEngineCvar007`.
3. For functions that aren't easily accessible via static vtable indexes, we use **Memory Pattern Scanning** (`signatures.rs`) to find the exact memory address of the function we want to call.
4. We cast these addresses to Rust `unsafe extern "thiscall"` function pointers and wrap them in safe methods.

## License

MIT License - see [LICENSE](LICENSE) for details. Built with ❤️ for the Portal 2 modding community.