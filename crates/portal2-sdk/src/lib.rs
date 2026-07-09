//! # Portal 2 Rust SDK (`portal2-sdk`)
//!
//! A high-level, safe, and ergonomic Rust library for interacting with the Source engine's internal C++ virtual
//! interfaces and subsystems in **Portal 2**.
//!
//! This crate utilizes dynamic memory scanning (pattern/signature matching) and virtual vtable borrowing to interact
//! directly with the game's core systems cleanly without raw C++ wrappers or hardcoded absolute pointers.
//!
//! ## Core Capabilities
//!
//! - **CVar & Command Management (`ICvar`)**: High-level builders (`ConVarBuilder`, `ConCommandBuilder`) to register custom console settings and commands with Rust callbacks.
//! - **Developer Console Printing**: Direct, formatted printing to the in-game developer console (`~`) with custom RGBA colors via `con_print!` and `con_color_print!`.
//! - **Engine & Server Interfaces**: Access to `IVEngineClient`, `IVEngineServer`, `IGameEventManager2`, entity systems, ray tracing, and debug overlays.
//!
//! ## Quick Start Examples
//!
//! ### 1. Printing to the Developer Console (`~`)
//!
//! ```rust,no_run
//! use portal2_sdk::{con_print, con_color_print, Color};
//!
//! // Print standard white text:
//! con_print!("Loaded mod version {}\n", 1.0);
//!
//! // Print colored status text:
//! con_color_print!(Color::rgb(0, 255, 0), "[OK] Initialization successful!\n");
//! con_color_print!(Color::rgb(255, 100, 0), "[WARN] High ping detected: {}ms\n", 150);
//! ```
//!
//! ### 2. Registering a Custom Console Command (`ConCommand`)
//!
//! ```rust,no_run
//! use portal2_sdk::{ConCommand, CCommand, CvarFlags, con_print, con_color_print, Color};
//!
//! extern "C" fn my_teleport_cmd(cmd: &CCommand) {
//!     if let Some(target) = cmd.arg(1) {
//!         con_color_print!(Color::rgb(0, 255, 255), "Teleporting to: {}\n", target);
//!     } else {
//!         con_print!("Usage: my_teleport <destination>\n");
///     }
/// }
///
/// fn setup_commands() {
///     ConCommand::register_new(
///         "my_teleport",
///         "Teleports the player to a target location",
///         CvarFlags::NONE,
///         my_teleport_cmd,
///     ).expect("Failed to register ConCommand");
/// }
/// ```
///
/// ### 3. Registering a Bounded Console Variable (`ConVar`)
///
/// ```rust,no_run
/// use portal2_sdk::{ConVar, CvarFlags};
///
/// fn setup_cvars() {
///     let fov = ConVar::builder("my_custom_fov", "90.0")
///         .help_text("Custom field of view setting")
///         .flags(CvarFlags::ARCHIVE)
///         .min(60.0)
///         .max(140.0)
///         .register()
///         .expect("Failed to register ConVar");
///
///     con_print!("Current custom FOV: {}", fov.get_float());
/// }
/// ```
use std::sync::OnceLock;
use std::{ffi::c_void, slice};
use std::sync::atomic::{AtomicBool, Ordering};

mod signatures;
pub mod types;
mod interfaces;
mod memory;
mod entities;
pub mod utils;

mod server;
mod server_tools;
mod client;
mod cvar;
pub mod input_system;
pub mod game_events;
pub mod engine_trace;
pub mod debug_overlay;

pub use crate::entities::Entities;
use crate::input_system::IInputStackSystem;
use crate::server::IVEngineServer;
use crate::server_tools::IServerTools;
pub use client::IVEngineClient;
pub use cvar::{ICvar, CvarFlags, ConVar, ConVarBuilder, ConCommand, ConCommandBuilder, ConCommandBase, CCommand, Color};
pub use game_events::IGameEventManager2;
pub use engine_trace::IEngineTrace;
pub use debug_overlay::IVDebugOverlay;

pub static ENGINE: OnceLock<Engine> = OnceLock::new();

pub fn get_engine() -> &'static Engine {
    ENGINE.get().expect("Engine not initialized!")
}

/// A struct that holds pointers to all the game engine interfaces we need.
pub struct Engine {
    client: IVEngineClient,
    input_stack_system: IInputStackSystem,
    icvar: ICvar,
    game_event_manager: IGameEventManager2,
    engine_server: IVEngineServer,
    engine_trace: IEngineTrace,
    debug_overlay: IVDebugOverlay,
    server_tools: OnceLock<IServerTools>,
}

/// SAFETY:
/// This implementation is safe under the assumption that this struct is written to only
/// ONCE during initialization from a single thread, and then only read from.
unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

/// Provides safe, ergonomic accessors to the interfaces.
impl Engine {
    pub fn client(&self) -> &IVEngineClient {
        &self.client
    }

    /// Returns an immutable reference to the IInputStackSystem interface.
    pub fn input_stack_system(&self) -> &IInputStackSystem {
        &self.input_stack_system
    }

    /// Returns an immutable reference to the ICvar interface.
    pub fn cvar_system(&self) -> &ICvar {
        &self.icvar
    }

    pub fn game_event_manager(&self) -> &IGameEventManager2 {
        &self.game_event_manager
    }

    pub fn engine_server(&self) -> &IVEngineServer {
        &self.engine_server
    }

    pub fn engine_trace(&self) -> &IEngineTrace {
        &self.engine_trace
    }

    pub fn debug_overlay(&self) -> &IVDebugOverlay {
        &self.debug_overlay
    }

    pub fn server_tools(&self) -> &IServerTools {
        if let Some(tools) = self.server_tools.get() {
            return tools;
        }

        // Okay, lets try init this now...
        if let Some(tools) = Self::initialize_server_tools() {
            let _ = self.server_tools.set(tools);
            if let Some(tools) = self.server_tools.get() {
                return tools;
            }
        }

        panic!("Failed to initialize IServerTools interface. Possibly called too early.")
    }

    pub fn entities(&self) -> Entities<'_> {
        Entities::new(self.server_tools())
    }
}

// A helper macro to reduce boilerplate when finding functions.
macro_rules! find_fn {
    ($mem_slice:expr, $base_addr:expr, $pattern:expr, $mask:expr, $fn_name:literal) => {
        match memory::find_pattern($mem_slice, $pattern, $mask) {
            Some(offset) => unsafe { std::mem::transmute($base_addr.add(offset)) },
            None => {
                return Err(format!("{} signature not found!", $fn_name));
            }
        }
    };
}
macro_rules! get_vfunc { // for unique cases
    ($this:expr, $idx:expr) => {
        unsafe {
            let vtable = *($this as *const *const usize);
            let func_ptr = vtable.add($idx).read();
            std::mem::transmute(func_ptr)
        }
    };
}


/// Initializes all engine interfaces by finding them in memory and resolving function pointers.
/// This is the core of the signature-based approach. It must be called once before `get()`.
impl Engine {
    pub fn initialize() -> Result<&'static Engine, String> {
        static INITED: AtomicBool = AtomicBool::new(false);
        if INITED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            return Err("Re-initialization is prohibited!".to_string());
        }

        // --- Get the base "this" pointers for each interface. ---
        // We use `find_interface` for this, as it's a reliable way to get the
        // object's address, which is required for all `thiscall` functions.
        let client_this = unsafe {
            interfaces::find_interface::<c_void>(b"engine.dll\0", b"VEngineClient015\0")
        };
        if client_this.is_null() {
            return Err("Failed to find IVEngineClient interface pointer.".to_string());
        }

        let input_stack_system_this = unsafe {
            interfaces::find_interface::<c_void>(b"inputsystem.dll\0", b"InputStackSystemVersion001\0")
        };
        if input_stack_system_this.is_null() {
            return Err("Failed to find IInputStackSystem interface pointer".to_string());
        }

        let icvar_this =
            unsafe { interfaces::find_interface::<c_void>(b"vstdlib.dll\0", b"VEngineCvar007\0") };
        if icvar_this.is_null() {
            return Err("Failed to find ICvar interface pointer".to_string());
        }

        let game_event_manager_this = unsafe {
            interfaces::find_interface::<c_void>(b"engine.dll\0", b"GAMEEVENTSMANAGER002\0")
        };
        if game_event_manager_this.is_null() {
            return Err("Failed to find IGameEventManager2 interface pointer".to_string());
        }

        let engine_server_this = unsafe {
            interfaces::find_interface::<c_void>(b"engine.dll\0", b"VEngineServer022\0")
        };
        if engine_server_this.is_null() {
            return Err("Failed to find IVEngineServer interface pointer.".to_string());
        }

        let engine_trace_this = unsafe {
            interfaces::find_interface::<c_void>(b"engine.dll\0", b"EngineTraceServer004\0")
        };
        if engine_trace_this.is_null() {
            return Err("Failed to find IEngineTrace interface pointer.".to_string());
        }

        let debug_overlay_this = unsafe {
            interfaces::find_interface::<c_void>(b"engine.dll\0", b"VDebugOverlay004\0")
        };
        if debug_overlay_this.is_null() {
            return Err("Failed to find IVDebugOverlay interface pointer.".to_string());
        }

        // --- Get the memory ranges of the modules to scan. ---
        let engine_dll = unsafe { memory::get_module_memory_range(b"engine.dll\0") };
        let inputsystem_dll = unsafe { memory::get_module_memory_range(b"inputsystem.dll\0") };
        let vstdlib_dll = unsafe { memory::get_module_memory_range(b"vstdlib.dll\0") };

        if engine_dll.is_none() || inputsystem_dll.is_none() || vstdlib_dll.is_none() {
            return Err("Failed to get one or more module memory ranges".to_string());
        }
        let (engine_base, engine_size) = engine_dll.unwrap();
        let engine_mem = unsafe { slice::from_raw_parts(engine_base, engine_size) };

        let (input_base, input_size) = inputsystem_dll.unwrap();
        let input_mem = unsafe { slice::from_raw_parts(input_base, input_size) };

        let (vstdlib_base, vstdlib_size) = vstdlib_dll.unwrap();
        let vstdlib_mem = unsafe { slice::from_raw_parts(vstdlib_base, vstdlib_size) };

        // --- Find function addresses using signatures and construct interface structs. ---

        use signatures::iv_engine_client::*;
        let client = IVEngineClient {
            this: client_this as *mut _,
            server_cmd:             find_fn!(engine_mem, engine_base, SERVER_CMD_PATTERN, SERVER_CMD_MASK, "ServerCmd"),
            client_cmd:             find_fn!(engine_mem, engine_base, CLIENT_CMD_PATTERN, CLIENT_CMD_MASK, "ClientCmd"),
            get_player_info:        find_fn!(engine_mem, engine_base, GET_PLAYER_INFO_PATTERN, GET_PLAYER_INFO_MASK, "GetPlayerInfo"),
            get_last_time_stamp:    find_fn!(engine_mem, engine_base, GET_LAST_TIME_STAMP_PATTERN, GET_LAST_TIME_STAMP_MASK, "GetLastTimeStamp"),
            get_view_angles:        find_fn!(engine_mem, engine_base, GET_VIEW_ANGLES_PATTERN, GET_VIEW_ANGLES_MASK, "GetViewAngles"),
            set_view_angles:        find_fn!(engine_mem, engine_base, SET_VIEW_ANGLES_PATTERN, SET_VIEW_ANGLES_MASK, "SetViewAngles"),
            is_in_game:             find_fn!(engine_mem, engine_base, IS_IN_GAME_PATTERN, IS_IN_GAME_MASK, "IsInGame"),
            is_connected:           find_fn!(engine_mem, engine_base, IS_CONNECTED_PATTERN, IS_CONNECTED_MASK, "IsConnected"),
            is_singlplayer:         find_fn!(engine_mem, engine_base, IS_SINGLPLAYER_PATTERN, IS_SINGLPLAYER_MASK, "IsSingleplayer"),
            get_screen_size:        find_fn!(engine_mem, engine_base, GET_SCREEN_SIZE_PATTERN, GET_SCREEN_SIZE_MASK, "GetScreenSize"),
            get_player_for_user_id: find_fn!(engine_mem, engine_base, GET_PLAYER_FOR_USER_ID_PATTERN, GET_PLAYER_FOR_USER_ID_MASK, "GetPlayerForUserId"),
            get_local_player:       find_fn!(engine_mem, engine_base, GET_LOCAL_PLAYER_PATTERN, GET_LOCAL_PLAYER_MASK, "GetLocalPlayer"),
            load_model:             find_fn!(engine_mem, engine_base, LOAD_MODEL_PATTERN, LOAD_MODEL_MASK, "LoadModel"),
            key_lookup_binding:     find_fn!(engine_mem, engine_base, KEY_LOOKUP_BINDING_PATTERN, KEY_LOOKUP_BINDING_MASK, "KeyLookupBinding"),
            execute_client_cmd_unrestricted:  find_fn!(engine_mem, engine_base, EXECUTE_CLIENT_CMD_UNRESTRICTED_PATTERN, EXECUTE_CLIENT_CMD_UNRESTRICTED_MASK, "ExecuteClientCmdUnrestricted"),


            // Unique cases of too short functions. They cannot be found by signature, using vtable indexes
            con_is_visible:             get_vfunc!(client_this, 11),
            get_max_clients:            get_vfunc!(client_this, 20),
            is_drawing_loading_image:   get_vfunc!(client_this, 27),
            get_level_name:             get_vfunc!(client_this, 52),
            get_level_name_short:       get_vfunc!(client_this, 53),
            is_paused:                  get_vfunc!(client_this, 86),
        };

        use signatures::iinput_stack_system::*;
        let input_stack_system = IInputStackSystem {
            this: input_stack_system_this as *mut _,
            push_input_context: find_fn!(input_mem, input_base, PUSH_INPUT_CONTEXT_PATTERN, PUSH_INPUT_CONTEXT_MASK, "PushInputContext"),
            enable_input_context: find_fn!(input_mem, input_base, ENABLE_INPUT_CONTEXT_PATTERN, ENABLE_INPUT_CONTEXT_MASK, "EnableInputContext"),
            set_cursor_visible: find_fn!(input_mem, input_base, SET_CURSOR_VISIBLE_PATTERN, SET_CURSOR_VISIBLE_MASK, "SetCursorVisible"),
            set_mouse_capture: find_fn!(input_mem, input_base, SET_MOUSE_CAPTURE_PATTERN, SET_MOUSE_CAPTURE_MASK, "SetMouseCapture"),
            set_cursor_position: find_fn!(input_mem, input_base, SET_CURSOR_POSITION_PATTERN, SET_CURSOR_POSITION_MASK, "SetCursorPosition"),
            is_topmost_enabled_context: find_fn!(input_mem, input_base, IS_TOPMOST_ENABLED_CONTEXT_PATTERN, IS_TOPMOST_ENABLED_CONTEXT_MASK, "IsTopmostEnabledContext"),
        };

        use signatures::icvar::*;
        let icvar = ICvar {
            this: icvar_this as *mut _,
            find_var: find_fn!(vstdlib_mem, vstdlib_base, FIND_VAR_PATTERN, FIND_VAR_MASK, "FindVar"),
            find_command_base: get_vfunc!(icvar_this, 13),
            register_con_command: get_vfunc!(icvar_this, 9),
            unregister_con_command: get_vfunc!(icvar_this, 10),
            console_color_printf: get_vfunc!(icvar_this, 24),
            console_printf: get_vfunc!(icvar_this, 25),
        };

        let game_event_manager = IGameEventManager2 {
            this: game_event_manager_this as *mut _,
            add_listener: get_vfunc!(game_event_manager_this, 3),
            remove_listener: get_vfunc!(game_event_manager_this, 5),
            listener: game_events::create_master_listener(),
        };

        let engine_server = IVEngineServer {
            this: engine_server_this as *mut _,
            change_level: get_vfunc!(engine_server_this, 0),
            is_map_valid: get_vfunc!(engine_server_this, 1),
            is_dedicated_server: get_vfunc!(engine_server_this, 2),
            is_in_edit_mode: get_vfunc!(engine_server_this, 3),
            get_launch_options: get_vfunc!(engine_server_this, 4),
            precache_model: get_vfunc!(engine_server_this, 5),
            precache_sentence_file: get_vfunc!(engine_server_this, 6),
            precache_decal: get_vfunc!(engine_server_this, 7),
            precache_generic: get_vfunc!(engine_server_this, 8),
            is_model_precached: get_vfunc!(engine_server_this, 9),
            is_decal_precached: get_vfunc!(engine_server_this, 10),
            is_generic_precached: get_vfunc!(engine_server_this, 11),
            get_cluster_for_origin: get_vfunc!(engine_server_this, 12),
            get_pvs_for_cluster: get_vfunc!(engine_server_this, 13),
            check_origin_in_pvs: get_vfunc!(engine_server_this, 14),
            check_box_in_pvs: get_vfunc!(engine_server_this, 15),
            get_player_user_id: get_vfunc!(engine_server_this, 16),
            get_player_network_id_string: get_vfunc!(engine_server_this, 17),
            is_user_id_in_use: get_vfunc!(engine_server_this, 18),
            get_loading_progress_for_user_id: get_vfunc!(engine_server_this, 19),
            get_entity_count: get_vfunc!(engine_server_this, 20),
            get_player_net_info: get_vfunc!(engine_server_this, 21),
            create_edict: get_vfunc!(engine_server_this, 22),
            remove_edict: get_vfunc!(engine_server_this, 23), // todo: invalid index?
            pv_alloc_ent_private_data: get_vfunc!(engine_server_this, 24),
            free_ent_private_data: get_vfunc!(engine_server_this, 25),
            save_alloc_memory: get_vfunc!(engine_server_this, 26),
            save_free_memory: get_vfunc!(engine_server_this, 27),
            emit_ambient_sound: get_vfunc!(engine_server_this, 28),
            fade_client_volume: get_vfunc!(engine_server_this, 29),
            sentence_group_pick: get_vfunc!(engine_server_this, 30),
            sentence_group_pick_sequential: get_vfunc!(engine_server_this, 31),
            sentence_index_from_name: get_vfunc!(engine_server_this, 32),
            sentence_name_from_index: get_vfunc!(engine_server_this, 33),
            sentence_group_index_from_name: get_vfunc!(engine_server_this, 34),
            sentence_group_name_from_index: get_vfunc!(engine_server_this, 35),
            sentence_length: get_vfunc!(engine_server_this, 36),
            server_command: get_vfunc!(engine_server_this, 37),
            server_execute: get_vfunc!(engine_server_this, 38),
            client_command: get_vfunc!(engine_server_this, 39),
            light_style: get_vfunc!(engine_server_this, 40),
            static_decal: get_vfunc!(engine_server_this, 41),
            message_determine_multicast_recipients: get_vfunc!(engine_server_this, 42),
            entity_message_begin: get_vfunc!(engine_server_this, 43),
            user_message_begin: get_vfunc!(engine_server_this, 44),
            message_end: get_vfunc!(engine_server_this, 45),
            client_printf: get_vfunc!(engine_server_this, 46),
            con_nprintf: get_vfunc!(engine_server_this, 47),
            // con_nxprintf: get_vfunc!(engine_server_this, 48),
            set_view: get_vfunc!(engine_server_this, 49),
            crosshair_angle: get_vfunc!(engine_server_this, 50),
            get_game_dir: get_vfunc!(engine_server_this, 51),
            compare_file_time: get_vfunc!(engine_server_this, 52),
            lock_network_string_tables: get_vfunc!(engine_server_this, 53),
            create_fake_client: get_vfunc!(engine_server_this, 54),
            get_client_con_var_value: get_vfunc!(engine_server_this, 55),
            parse_file: get_vfunc!(engine_server_this, 56),
            copy_file: get_vfunc!(engine_server_this, 57),
            reset_pvs: get_vfunc!(engine_server_this, 58),
            add_origin_to_pvs: get_vfunc!(engine_server_this, 59),
            set_area_portal_state: get_vfunc!(engine_server_this, 60),
            playback_temp_entity: get_vfunc!(engine_server_this, 61),
            check_headnode_visible: get_vfunc!(engine_server_this, 62),
            check_areas_connected: get_vfunc!(engine_server_this, 63),
            get_area: get_vfunc!(engine_server_this, 64),
            get_area_bits: get_vfunc!(engine_server_this, 65),
            get_area_portal_plane: get_vfunc!(engine_server_this, 66),
            load_game_state: get_vfunc!(engine_server_this, 67),
            load_adjacent_ents: get_vfunc!(engine_server_this, 68),
            clear_save_dir: get_vfunc!(engine_server_this, 69),
            get_map_entities_string: get_vfunc!(engine_server_this, 70),
            text_message_get: get_vfunc!(engine_server_this, 71),
            log_print: get_vfunc!(engine_server_this, 72),
            is_log_enabled: get_vfunc!(engine_server_this, 73),
            build_entity_cluster_list: get_vfunc!(engine_server_this, 74),
            solid_moved: get_vfunc!(engine_server_this, 75),
            trigger_moved: get_vfunc!(engine_server_this, 76),
            create_spatial_partition: get_vfunc!(engine_server_this, 77),
            destroy_spatial_partition: get_vfunc!(engine_server_this, 78),
            draw_map_to_scratch_pad: get_vfunc!(engine_server_this, 79),
            get_entity_transmit_bits_for_client: get_vfunc!(engine_server_this, 80),
            is_paused: get_vfunc!(engine_server_this, 81),
            get_timescale: get_vfunc!(engine_server_this, 82),
            force_exact_file: get_vfunc!(engine_server_this, 83),
            force_model_bounds: get_vfunc!(engine_server_this, 84),
            clear_save_dir_after_client_load: get_vfunc!(engine_server_this, 85),
            set_fake_client_con_var_value: get_vfunc!(engine_server_this, 86),
            force_simple_material: get_vfunc!(engine_server_this, 87),
            is_in_commentary_mode: get_vfunc!(engine_server_this, 88),
            is_level_main_menu_background: get_vfunc!(engine_server_this, 89),
            set_area_portal_states: get_vfunc!(engine_server_this, 90),
            notify_edict_flags_change: get_vfunc!(engine_server_this, 91),
            get_prev_check_transmit_info: get_vfunc!(engine_server_this, 92),
            get_shared_edict_change_info: get_vfunc!(engine_server_this, 93),
            allow_immediate_edict_reuse: get_vfunc!(engine_server_this, 94),
            is_internal_build: get_vfunc!(engine_server_this, 95),
            get_change_accessor: get_vfunc!(engine_server_this, 96),
            get_most_recently_loaded_file_name: get_vfunc!(engine_server_this, 97),
            get_save_file_name: get_vfunc!(engine_server_this, 98),
            clean_up_entity_cluster_list: get_vfunc!(engine_server_this, 99),
            get_app_id: get_vfunc!(engine_server_this, 100),
            is_low_violence: get_vfunc!(engine_server_this, 101),
            is_any_client_low_violence: get_vfunc!(engine_server_this, 102),
            start_query_cvar_value: get_vfunc!(engine_server_this, 103),
            insert_server_command: get_vfunc!(engine_server_this, 104),
            get_player_info: get_vfunc!(engine_server_this, 105),
            is_client_fully_authenticated: get_vfunc!(engine_server_this, 106),
            set_dedicated_server_benchmark_mode: get_vfunc!(engine_server_this, 107),
            is_split_screen_player: get_vfunc!(engine_server_this, 108),
            get_split_screen_player_attach_to_edict: get_vfunc!(engine_server_this, 109),
            get_num_split_screen_users_attached_to_edict: get_vfunc!(engine_server_this, 110),
            get_split_screen_player_for_edict: get_vfunc!(engine_server_this, 111),
            is_override_load_game_ents_on: get_vfunc!(engine_server_this, 112),
            force_flush_entity: get_vfunc!(engine_server_this, 113),
            get_single_player_shared_memory_space: get_vfunc!(engine_server_this, 114),
            alloc_level_static_data: get_vfunc!(engine_server_this, 115),
            get_cluster_count: get_vfunc!(engine_server_this, 116),
            get_all_cluster_bounds: get_vfunc!(engine_server_this, 117),
            is_creating_reslist: get_vfunc!(engine_server_this, 118),
            is_creating_xbox_reslist: get_vfunc!(engine_server_this, 119),
            is_dedicated_server_for_xbox: get_vfunc!(engine_server_this, 120),
            pause: get_vfunc!(engine_server_this, 121),
            set_timescale: get_vfunc!(engine_server_this, 122),
            set_gamestats_data: get_vfunc!(engine_server_this, 123),
            get_gamestats_data: get_vfunc!(engine_server_this, 124),
            get_client_steam_id: get_vfunc!(engine_server_this, 125),
            get_game_server_steam_id: get_vfunc!(engine_server_this, 126),
            host_validate_session: get_vfunc!(engine_server_this, 127),
            refresh_screen_if_necessary: get_vfunc!(engine_server_this, 128),
            has_paintmap: get_vfunc!(engine_server_this, 129),
            sphere_paint_surface: get_vfunc!(engine_server_this, 130),
            sphere_trace_paint_surface: get_vfunc!(engine_server_this, 131),
            remove_all_paint: get_vfunc!(engine_server_this, 132),
            paint_all_surfaces: get_vfunc!(engine_server_this, 133),
            remove_paint: get_vfunc!(engine_server_this, 134),
            client_command_key_values: get_vfunc!(engine_server_this, 135),
            get_client_xuid: get_vfunc!(engine_server_this, 136),
            is_active_app: get_vfunc!(engine_server_this, 137),
            set_no_clip_enabled: get_vfunc!(engine_server_this, 138),
            get_paintmap_data_rle: get_vfunc!(engine_server_this, 139),
            load_paintmap_data_rle: get_vfunc!(engine_server_this, 140),
            send_paintmap_data_to_client: get_vfunc!(engine_server_this, 141),
            get_latency_for_choreo_sounds: get_vfunc!(engine_server_this, 142),
            get_client_cross_play_platform: get_vfunc!(engine_server_this, 143),
        };

        let engine_trace = IEngineTrace {
            this: engine_trace_this as *mut _,
            get_point_contents: get_vfunc!(engine_trace_this, 0),
            clip_ray_to_entity: get_vfunc!(engine_trace_this, 3),
            trace_ray:          get_vfunc!(engine_trace_this, 5),
            get_collideable:    get_vfunc!(engine_trace_this, 12),
        };

        let debug_overlay = IVDebugOverlay {
            this: debug_overlay_this as *mut _,
            add_box_overlay: get_vfunc!(debug_overlay_this, 1),
            add_sphere_overlay: get_vfunc!(debug_overlay_this, 2),
            add_line_overlay: get_vfunc!(debug_overlay_this, 4),
            add_text_overlay: get_vfunc!(debug_overlay_this, 5),
            add_screen_text_overlay: get_vfunc!(debug_overlay_this, 7),
            screen_position: get_vfunc!(debug_overlay_this, 12),
            clear_all_overlays: get_vfunc!(debug_overlay_this, 16),
        };

        let server_tools = OnceLock::new();
        if let Some(st) = Self::initialize_server_tools() {
            let _ = server_tools.set(st);
        }

        let engine = Engine {
            client,
            input_stack_system,
            icvar,
            game_event_manager,
            engine_server,
            engine_trace,
            debug_overlay,
            server_tools,
        };

        ENGINE.set(engine).map_err(|_| "Engine already initialized!")?;
        Ok(&ENGINE.get().unwrap())
    }

    fn initialize_server_tools() -> Option<IServerTools> {
        let server_tools_this = unsafe {
            interfaces::find_interface::<c_void>(b"server.dll\0", b"VSERVERTOOLS001\0")
        };
        if server_tools_this.is_null() {
            return None;
        }
        let server_tools = IServerTools {
            this: server_tools_this as *mut _,
            get_iserver_entity: get_vfunc!(server_tools_this, 1),
            snap_player_to_position: get_vfunc!(server_tools_this, 2),
            get_player_position: get_vfunc!(server_tools_this, 3),
            set_player_fov: get_vfunc!(server_tools_this, 4),
            get_player_fov: get_vfunc!(server_tools_this, 5),
            is_in_no_clip_mode: get_vfunc!(server_tools_this, 6),
            first_entity: get_vfunc!(server_tools_this, 7),
            next_entity: get_vfunc!(server_tools_this, 8),
            find_entity_by_hammer_id: get_vfunc!(server_tools_this, 9),
            get_key_value: get_vfunc!(server_tools_this, 10),
            set_key_value_vec: get_vfunc!(server_tools_this, 11),
            set_key_value_flt: get_vfunc!(server_tools_this, 12),
            set_key_value_str: get_vfunc!(server_tools_this, 13),
            create_entity_by_name: get_vfunc!(server_tools_this, 14),
            dispatch_spawn: get_vfunc!(server_tools_this, 15),
            destroy_entity_by_hammer_id: get_vfunc!(server_tools_this, 16),
            respawn_entities_with_edits: get_vfunc!(server_tools_this, 17),
            reload_particle_defintions: get_vfunc!(server_tools_this, 18),
            add_origin_to_pvs: get_vfunc!(server_tools_this, 19),
            move_engine_view_to: get_vfunc!(server_tools_this, 20),
            remove_entity: get_vfunc!(server_tools_this, 21),
        };

        Some(server_tools)
    }
}

/// Prints a formatted message directly to the in-game developer console (`~`).
#[macro_export]
macro_rules! con_print {
    ($($arg:tt)*) => {
        $crate::console_print(&std::format!($($arg)*))
    };
}

/// Prints a colored formatted message directly to the in-game developer console (`~`).
///
/// Accepts a `Color` (RGBA/RGB) struct as the first argument, followed by standard `format!` arguments.
///
/// # Example
///
/// ```rust,no_run
/// use portal2_sdk::{con_color_print, Color};
///
/// con_color_print!(Color::rgb(0, 255, 0), "[SUCCESS] Plugin enabled successfully.\n");
/// con_color_print!(Color::rgb(255, 50, 50), "[ERROR] Failed to read config code {}\n", 404);
/// ```
#[macro_export]
macro_rules! con_color_print {
    ($color:expr, $($arg:tt)*) => {
        $crate::console_color_print($color, &std::format!($($arg)*))
    };
}

/// Prints a raw string directly to the in-game developer console (`~`).
pub fn console_print(msg: &str) {
    if let Some(engine) = ENGINE.get() {
        engine.cvar_system().console_print(msg);
    }
}

/// Prints a raw colored (`RGBA`) string directly to the in-game developer console (`~`).
pub fn console_color_print(color: Color, msg: &str) {
    if let Some(engine) = ENGINE.get() {
        engine.cvar_system().console_color_print(color, msg);
    }
}
