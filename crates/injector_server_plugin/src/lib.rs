//! plugin_d3d9_hook: Source server plugin that initializes overlay via offsets.
//!
//! - Implements IServerPluginCallbacks interface.
//! - On load (or server_activate) starts the offsets hook thread.
//! - Returns the plugin instance via CreateInterface("ISERVERPLUGINCALLBACKS003").

#![cfg(all(target_os = "windows", target_pointer_width = "32"))]

use std::ffi::{c_char, c_void};
use std::ptr::null_mut;

type CreateInterfaceFn = unsafe extern "C" fn(name: *const c_char, return_code: *mut i32) -> *mut c_void;

#[repr(C)]
struct IServerPluginCallbacksVtable {
    load: unsafe extern "thiscall" fn(this: *mut c_void, interface_factory: CreateInterfaceFn, game_server_factory: CreateInterfaceFn) -> bool,
    unload: unsafe extern "thiscall" fn(this: *mut c_void),
    pause: unsafe extern "thiscall" fn(this: *mut c_void),
    unpause: unsafe extern "thiscall" fn(this: *mut c_void),
    get_plugin_description: unsafe extern "thiscall" fn(this: *mut c_void) -> *const c_char,
    level_init: unsafe extern "thiscall" fn(this: *mut c_void, map_name: *const c_char),
    server_activate: unsafe extern "thiscall" fn(this: *mut c_void, entity_list: *const c_void, entity_count: i32, client_max: i32),
    game_frame: unsafe extern "thiscall" fn(this: *mut c_void, simulating: bool),
    level_shutdown: unsafe extern "thiscall" fn(this: *mut c_void),
    client_active: unsafe extern "thiscall" fn(this: *mut c_void, entity: *const c_void),
    client_fully_connect: unsafe extern "thiscall" fn(this: *mut c_void, entity: *const c_void),
    client_disconnect: unsafe extern "thiscall" fn(this: *mut c_void, entity: *const c_void),
    client_put_in_server: unsafe extern "thiscall" fn(this: *mut c_void, entity: *const c_void, player_name: *const c_char),
    set_command_client: unsafe extern "thiscall" fn(this: *mut c_void, index: i32),
    client_settings_changed: unsafe extern "thiscall" fn(this: *mut c_void, entity: *const c_void),
    client_connect: unsafe extern "thiscall" fn(this: *mut c_void, allow_connect: *mut bool, entity: *const c_void, name: *const c_char, address: *const c_char, reject: *mut c_char, reject_len: i32) -> i32,
    client_command: unsafe extern "thiscall" fn(this: *mut c_void, entity: *const c_void, args: *const c_void) -> i32,
    network_id_validated: unsafe extern "thiscall" fn(this: *mut c_void, username: *const c_char, network_id: *const c_char) -> i32,
    on_query_cvar_value_finished: unsafe extern "thiscall" fn(this: *mut c_void, cookie: i32, entity: *const c_void, status: i32, cvar_name: *const c_char, cvar_value: *const c_char),
    on_edict_allocated: unsafe extern "thiscall" fn(this: *mut c_void, entity: *const c_void),
    on_edict_freed: unsafe extern "thiscall" fn(this: *mut c_void, entity: *const c_void),
}

#[repr(C)]
struct ServerPlugin {
    vtable: *const IServerPluginCallbacksVtable,
}

unsafe extern "thiscall" fn plugin_load(_this: *mut c_void, _interface_factory: CreateInterfaceFn, _game_server_factory: CreateInterfaceFn) -> bool {
    overlay_runtime::logger::init();

    d3d9_hook_core::start_offsets_hook_thread(
        &[0xDA5D8usize, 0x179F38usize],
        500,
        &overlay_runtime::CALLBACKS,
    );
    true
}

unsafe extern "thiscall" fn plugin_unload(_this: *mut c_void) {
    // Uninstall D3D9 hooks (restore vtable entries)
    let _ = d3d9_hook_core::uninstall();

    // Restore WndProc and detach overlay glue
    overlay_runtime::uninstall_overlay();
}

unsafe extern "thiscall" fn server_activate(_this: *mut c_void, _entity_list: *const c_void, _entity_count: i32, _client_max: i32) {}
unsafe extern "thiscall" fn plugin_pause(_this: *mut c_void) {}
unsafe extern "thiscall" fn plugin_unpause(_this: *mut c_void) {}
unsafe extern "thiscall" fn level_init(_this: *mut c_void, _map_name: *const c_char) {}
unsafe extern "thiscall" fn game_frame(_this: *mut c_void, _simulating: bool) {}
unsafe extern "thiscall" fn level_shutdown(_this: *mut c_void) {}
unsafe extern "thiscall" fn client_active(_this: *mut c_void, _entity: *const c_void) {}
unsafe extern "thiscall" fn client_fully_connect(_this: *mut c_void, _entity: *const c_void) {}
unsafe extern "thiscall" fn client_disconnect(_this: *mut c_void, _entity: *const c_void) {}
unsafe extern "thiscall" fn client_put_in_server(_this: *mut c_void, _entity: *const c_void, _player_name: *const c_char) {}
unsafe extern "thiscall" fn set_command_client(_this: *mut c_void, _index: i32) {}
unsafe extern "thiscall" fn client_settings_changed(_this: *mut c_void, _entity: *const c_void) {}
unsafe extern "thiscall" fn client_connect(_this: *mut c_void, _allow_connect: *mut bool, _entity: *const c_void, _name: *const c_char, _address: *const c_char, _reject: *mut c_char, _reject_len: i32) -> i32 { 0 }
unsafe extern "thiscall" fn client_command(_this: *mut c_void, _entity: *const c_void, _args: *const c_void) -> i32 { 0 }
unsafe extern "thiscall" fn network_id_validated(_this: *mut c_void, _username: *const c_char, _network_id: *const c_char) -> i32 { 0 }
unsafe extern "thiscall" fn on_query_cvar_value_finished(_this: *mut c_void, _cookie: i32, _entity: *const c_void, _status: i32, _cvar_name: *const c_char, _cvar_value: *const c_char) {}
unsafe extern "thiscall" fn on_edict_allocated(_this: *mut c_void, _entity: *const c_void) {}
unsafe extern "thiscall" fn on_edict_freed(_this: *mut c_void, _entity: *const c_void) {}

static PLUGIN_VTABLE: IServerPluginCallbacksVtable = IServerPluginCallbacksVtable {
    load: plugin_load, unload: plugin_unload, pause: plugin_pause, unpause: plugin_unpause,
    get_plugin_description: get_plugin_description, level_init, server_activate, game_frame, level_shutdown,
    client_active, client_fully_connect, client_disconnect, client_put_in_server, set_command_client, client_settings_changed,
    client_connect, client_command, network_id_validated, on_query_cvar_value_finished, on_edict_allocated, on_edict_freed,
};

static mut SERVER_PLUGIN: ServerPlugin = ServerPlugin { vtable: &PLUGIN_VTABLE };

static PLUGIN_DESCRIPTION: &[u8] = b"Rust D3D9 Hook Plugin\0";
unsafe extern "thiscall" fn get_plugin_description(_this: *mut c_void) -> *const c_char {
    PLUGIN_DESCRIPTION.as_ptr() as *const c_char
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
#[allow(static_mut_refs)]
pub unsafe extern "C" fn CreateInterface(name: *const c_char, return_code: *mut i32) -> *mut c_void { unsafe {
    let requested_interface = std::ffi::CStr::from_ptr(name).to_string_lossy();
    if requested_interface == "ISERVERPLUGINCALLBACKS003" {
        if !return_code.is_null() { *return_code = 0; }
        return &mut SERVER_PLUGIN as *mut _ as *mut c_void;
    }
    if !return_code.is_null() { *return_code = 1; }
    null_mut()
}}
