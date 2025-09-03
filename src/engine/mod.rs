mod signatures;
pub mod types;
mod interfaces;

mod client;
mod convar;
pub mod input_system;
// mod vscript;

pub use client::IVEngineClient;

use std::{ffi::c_void, slice, sync::OnceLock};

use crate::{engine::{convar::ICvar, input_system::IInputStackSystem}, memory};

/// A struct that holds pointers to all the game engine interfaces we need.
pub struct Engine {
    client: IVEngineClient,
    input_stack_system: IInputStackSystem,
    icvar: ICvar,
    // todo mb add more interfaces here later, e.g.:
    // pub entity_list: SendablePtr<IClientEntityList>,
}

/// # Safety
/// This implementation is safe under the assumption that this struct is written to only
/// ONCE during initialization from a single thread, and then only read from.
/// The `OnceLock` guarantees the single-write behavior. The raw pointers inside the
/// interface structs (`this` and function pointers) are then constant and can be safely
/// accessed from multiple threads (assuming the underlying game functions are thread-safe).
unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}

/// A global, thread-safe, one-time initialized container for our engine interfaces.
static ENGINE: OnceLock<Engine> = OnceLock::new(); // todo: i can use mutex here, if if mutable behavior appears in the future


/// Initializes all engine interfaces by finding them in memory and resolving function pointers.
/// This is the core of the signature-based approach. It must be called once before `get()`.
///
/// # Returns
///
/// `true` if all interfaces and functions were successfully found, `false` otherwise.
pub fn initialize() -> bool {
    // --- Get the base "this" pointers for each interface. ---
    // We still use `find_interface` for this, as it's a reliable way to get the
    // object's address, which is required for all `thiscall` functions.
    let client_this = unsafe {
        interfaces::find_interface::<c_void>(b"engine.dll\0", b"VEngineClient015\0")
    };
    if client_this.is_null() {
        log::error!("[MOD ERROR] Failed to find IVEngineClient interface pointer.");
        return false;
    }

    let input_stack_system_this = unsafe {
        interfaces::find_interface::<c_void>(b"inputsystem.dll\0", b"InputStackSystemVersion001\0")
    };
    if input_stack_system_this.is_null() {
        log::error!("[MOD ERROR] Failed to find IInputStackSystem interface pointer.");
        return false;
    }

    let icvar_this =
        unsafe { interfaces::find_interface::<c_void>(b"vstdlib.dll\0", b"VEngineCvar007\0") };
    if icvar_this.is_null() {
        log::error!("[MOD ERROR] Failed to find ICvar interface pointer.");
        return false;
    }

    // --- Get the memory ranges of the modules to scan. ---
    let engine_dll = unsafe { memory::get_module_memory_range(b"engine.dll\0") };
    let inputsystem_dll = unsafe { memory::get_module_memory_range(b"inputsystem.dll\0") };
    let vstdlib_dll = unsafe { memory::get_module_memory_range(b"vstdlib.dll\0") };

    if engine_dll.is_none() || inputsystem_dll.is_none() || vstdlib_dll.is_none() {
        log::error!("[MOD ERROR] Failed to get one or more module memory ranges.");
        return false;
    }
    let (engine_base, engine_size) = engine_dll.unwrap();
    let engine_mem = unsafe { slice::from_raw_parts(engine_base, engine_size) };

    let (input_base, input_size) = inputsystem_dll.unwrap();
    let input_mem = unsafe { slice::from_raw_parts(input_base, input_size) };

    let (vstdlib_base, vstdlib_size) = vstdlib_dll.unwrap();
    let vstdlib_mem = unsafe { slice::from_raw_parts(vstdlib_base, vstdlib_size) };

    // --- Find function addresses using signatures and construct interface structs. ---

    // A helper macro to reduce boilerplate when finding functions.
    macro_rules! find_fn {
        ($mem_slice:expr, $base_addr:expr, $pattern:expr, $mask:expr, $fn_name:literal) => {
            match memory::find_pattern($mem_slice, $pattern, $mask) {
                Some(offset) => unsafe { std::mem::transmute($base_addr.add(offset)) },
                None => {
                    log::error!(concat!("[MOD ERROR] ", $fn_name, " signature not found!"));
                    return false;
                }
            }
        };
    }

    use signatures::iv_engine_client::*;
    let client = IVEngineClient {
        this: client_this as *mut _,
        server_cmd: find_fn!(engine_mem, engine_base, SERVER_CMD_PATTERN, SERVER_CMD_MASK, "ServerCmd"),
        client_cmd: find_fn!(engine_mem, engine_base, CLIENT_CMD_PATTERN, CLIENT_CMD_MASK, "ClientCmd"),
        get_player_info: find_fn!(engine_mem, engine_base, GET_PLAYER_INFO_PATTERN, GET_PLAYER_INFO_MASK, "GetPlayerInfo"),
        get_last_time_stamp: find_fn!(engine_mem, engine_base, GET_LAST_TIME_STAMP_PATTERN, GET_LAST_TIME_STAMP_MASK, "GetLastTimeStamp"),
        get_view_angles: find_fn!(engine_mem, engine_base, GET_VIEW_ANGLES_PATTERN, GET_VIEW_ANGLES_MASK, "GetViewAngles"),
        set_view_angles: find_fn!(engine_mem, engine_base, SET_VIEW_ANGLES_PATTERN, SET_VIEW_ANGLES_MASK, "SetViewAngles"),
        get_max_clients: find_fn!(engine_mem, engine_base, GET_MAX_CLIENTS_PATTERN, GET_MAX_CLIENTS_MASK, "GetMaxClients"),
        is_in_game: find_fn!(engine_mem, engine_base, IS_IN_GAME_PATTERN, IS_IN_GAME_MASK, "IsInGame"),
        is_connected: find_fn!(engine_mem, engine_base, IS_CONNECTED_PATTERN, IS_CONNECTED_MASK, "IsConnected"),
        is_drawing_loading_image: find_fn!(engine_mem, engine_base, IS_DRAWING_LOADING_IMAGE_PATTERN, IS_DRAWING_LOADING_IMAGE_MASK, "IsDrawingLoadingImage"),
        get_level_name: find_fn!(engine_mem, engine_base, GET_LEVEL_NAME_PATTERN, GET_LEVEL_NAME_MASK, "GetLevelName"),
        execute_client_cmd_unrestricted: find_fn!(engine_mem, engine_base, EXECUTE_CLIENT_CMD_UNRESTRICTED_PATTERN, EXECUTE_CLIENT_CMD_UNRESTRICTED_MASK, "ExecuteClientCmdUnrestricted"),
        is_singlplayer: find_fn!(engine_mem, engine_base, IS_SINGLPLAYER_PATTERN, IS_SINGLPLAYER_MASK, "IsSingleplayer"),
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
    };

    // --- Step 4: Store the fully constructed Engine struct in our static OnceLock. ---
    ENGINE
        .set(Engine {
            client,
            input_stack_system,
            icvar,
        })
        .is_ok()
}

/// Provides global, safe access to the initialized engine interfaces.
/// Panics if `initialize()` has not been called successfully.
pub fn get() -> &'static Engine {
    ENGINE.get().expect(
        "The engine module has not been initialized. Please call engine::initialize() first.",
    )
}

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
}
