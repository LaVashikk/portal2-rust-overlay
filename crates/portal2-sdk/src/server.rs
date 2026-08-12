#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;

use crate::types::*;
use crate::platform::abi::vfn;

pub struct IVEngineServer {
    pub(crate) this: *mut c_void,

    pub(crate) change_level: vfn!((this: *mut c_void, s1: *const c_char, s2: *const c_char)),
    pub(crate) is_map_valid: vfn!((this: *mut c_void, filename: *const c_char) -> c_int),
    pub(crate) is_dedicated_server: vfn!((this: *mut c_void) -> bool),
    pub(crate) is_in_edit_mode: vfn!((this: *mut c_void) -> c_int),
    pub(crate) get_launch_options: vfn!((this: *mut c_void) -> *mut KeyValues),
    pub(crate) precache_model: vfn!((this: *mut c_void, s: *const c_char, preload: bool) -> c_int),
    pub(crate) precache_sentence_file: vfn!((this: *mut c_void, s: *const c_char, preload: bool) -> c_int),
    pub(crate) precache_decal: vfn!((this: *mut c_void, name: *const c_char, preload: bool) -> c_int),
    pub(crate) precache_generic: vfn!((this: *mut c_void, s: *const c_char, preload: bool) -> c_int),
    pub(crate) is_model_precached: vfn!((this: *mut c_void, s: *const c_char) -> bool),
    pub(crate) is_decal_precached: vfn!((this: *mut c_void, s: *const c_char) -> bool),
    pub(crate) is_generic_precached: vfn!((this: *mut c_void, s: *const c_char) -> bool),
    pub(crate) get_cluster_for_origin: vfn!((this: *mut c_void, org: *const Vector) -> c_int),
    pub(crate) get_pvs_for_cluster: vfn!((this: *mut c_void, cluster: c_int, outputpvslength: c_int, outputpvs: *mut u8) -> c_int),
    pub(crate) check_origin_in_pvs: vfn!((this: *mut c_void, org: *const Vector, checkpvs: *const u8, checkpvssize: c_int) -> bool),
    pub(crate) check_box_in_pvs: vfn!((this: *mut c_void, mins: *const Vector, maxs: *const Vector, checkpvs: *const u8, checkpvssize: c_int) -> bool),
    pub(crate) get_player_user_id: vfn!((this: *mut c_void, e: *const Edict) -> c_int),
    pub(crate) get_player_network_id_string: vfn!((this: *mut c_void, e: *const Edict) -> *const c_char),
    pub(crate) is_user_id_in_use: vfn!((this: *mut c_void, user_id: c_int) -> bool),
    pub(crate) get_loading_progress_for_user_id: vfn!((this: *mut c_void, user_id: c_int) -> c_int),
    pub(crate) get_entity_count: vfn!((this: *mut c_void) -> c_int),
    pub(crate) get_player_net_info: vfn!((this: *mut c_void, player_index: c_int) -> *mut INetChannelInfo),
    pub(crate) create_edict: vfn!((this: *mut c_void, force_edict_index: c_int) -> *mut Edict),
    pub(crate) remove_edict: vfn!((this: *mut c_void, e: *mut Edict)),
    pub(crate) pv_alloc_ent_private_data: vfn!((this: *mut c_void, cb: c_int) -> *mut c_void),
    pub(crate) free_ent_private_data: vfn!((this: *mut c_void, entity: *mut c_void)),
    pub(crate) save_alloc_memory: vfn!((this: *mut c_void, num: usize, size: usize) -> *mut c_void),
    pub(crate) save_free_memory: vfn!((this: *mut c_void, save_mem: *mut c_void)),
    pub(crate) emit_ambient_sound: vfn!((this: *mut c_void, entindex: c_int, pos: *const Vector, samp: *const c_char, vol: f32, soundlevel: SoundLevelT, flags: c_int, pitch: c_int, delay: f32)),
    pub(crate) fade_client_volume: vfn!((this: *mut c_void, edict: *const Edict, fade_percent: f32, fade_out_seconds: f32, hold_time: f32, fade_in_seconds: f32)),
    pub(crate) sentence_group_pick: vfn!((this: *mut c_void, group_index: c_int, name: *mut c_char, name_buf_len: c_int) -> c_int),
    pub(crate) sentence_group_pick_sequential: vfn!((this: *mut c_void, group_index: c_int, name: *mut c_char, name_buf_len: c_int, sentence_index: c_int, reset: c_int) -> c_int),
    pub(crate) sentence_index_from_name: vfn!((this: *mut c_void, sentence_name: *const c_char) -> c_int),
    pub(crate) sentence_name_from_index: vfn!((this: *mut c_void, sentence_index: c_int) -> *const c_char),
    pub(crate) sentence_group_index_from_name: vfn!((this: *mut c_void, group_name: *const c_char) -> c_int),
    pub(crate) sentence_group_name_from_index: vfn!((this: *mut c_void, group_index: c_int) -> *const c_char),
    pub(crate) sentence_length: vfn!((this: *mut c_void, sentence_index: c_int) -> f32),
    pub(crate) server_command: vfn!((this: *mut c_void, str: *const c_char)),
    pub(crate) server_execute: vfn!((this: *mut c_void)),
    pub(crate) client_command: unsafe extern "C" fn(this: *mut c_void, edict: *mut Edict, fmt: *const c_char, ...), // Variadic (cdecl)
    pub(crate) light_style: vfn!((this: *mut c_void, style: c_int, val: *const c_char)),
    pub(crate) static_decal: vfn!((this: *mut c_void, origin: *const Vector, decal_index: c_int, entity_index: c_int, model_index: c_int, lowpriority: bool)),
    pub(crate) message_determine_multicast_recipients: vfn!((this: *mut c_void, usepas: bool, origin: *const Vector, playerbits: *mut CPlayerBitVec)),
    pub(crate) entity_message_begin: vfn!((this: *mut c_void, ent_index: c_int, ent_class: *mut ServerClass, reliable: bool) -> *mut BfWrite),
    pub(crate) user_message_begin: vfn!((this: *mut c_void, filter: *mut IRecipientFilter, msg_type: c_int, msg_name: *const c_char) -> *mut BfWrite),
    pub(crate) message_end: vfn!((this: *mut c_void)),
    pub(crate) client_printf: vfn!((this: *mut c_void, edict: *mut Edict, msg: *const c_char)),
    pub(crate) con_nprintf: unsafe extern "C" fn(this: *mut c_void, pos: c_int, fmt: *const c_char, ...), // Variadic (cdecl)
    // pub(crate) con_nxprintf: unsafe extern "C" fn(this: *mut c_void, info: *const ConNPrintS, fmt: *const c_char, ...), // Variadic (cdecl)
    pub(crate) set_view: vfn!((this: *mut c_void, client: *const Edict, viewent: *const Edict)),
    pub(crate) crosshair_angle: vfn!((this: *mut c_void, client: *const Edict, pitch: f32, yaw: f32)),
    pub(crate) get_game_dir: vfn!((this: *mut c_void, buf: *mut c_char, maxlen: c_int)),
    pub(crate) compare_file_time: vfn!((this: *mut c_void, filename1: *const c_char, filename2: *const c_char, compare: *mut c_int) -> c_int),
    pub(crate) lock_network_string_tables: vfn!((this: *mut c_void, lock: bool) -> bool),
    pub(crate) create_fake_client: vfn!((this: *mut c_void, netname: *const c_char) -> *mut Edict),
    pub(crate) get_client_con_var_value: vfn!((this: *mut c_void, client_index: c_int, name: *const c_char) -> *const c_char),
    pub(crate) parse_file: vfn!((this: *mut c_void, data: *const c_char, token: *mut c_char, maxlen: c_int) -> *const c_char),
    pub(crate) copy_file: vfn!((this: *mut c_void, source: *const c_char, dest: *const c_char) -> bool),
    pub(crate) reset_pvs: vfn!((this: *mut c_void, pvs: *mut u8, pvssize: c_int)),
    pub(crate) add_origin_to_pvs: vfn!((this: *mut c_void, origin: *const Vector)),
    pub(crate) set_area_portal_state: vfn!((this: *mut c_void, portal_number: c_int, is_open: c_int)),
    pub(crate) playback_temp_entity: vfn!((this: *mut c_void, filter: *mut IRecipientFilter, delay: f32, sender: *const c_void, st: *const SendTable, class_id: c_int)),
    pub(crate) check_headnode_visible: vfn!((this: *mut c_void, nodenum: c_int, pvs: *const u8, vissize: c_int) -> c_int),
    pub(crate) check_areas_connected: vfn!((this: *mut c_void, area1: c_int, area2: c_int) -> c_int),
    pub(crate) get_area: vfn!((this: *mut c_void, origin: *const Vector) -> c_int),
    pub(crate) get_area_bits: vfn!((this: *mut c_void, area: c_int, bits: *mut u8, buflen: c_int)),
    pub(crate) get_area_portal_plane: vfn!((this: *mut c_void, view_origin: *const Vector, portal_key: c_int, plane: *mut VPlane) -> bool),
    pub(crate) load_game_state: vfn!((this: *mut c_void, map_name: *const c_char, create_players: bool) -> bool),
    pub(crate) load_adjacent_ents: vfn!((this: *mut c_void, old_level: *const c_char, landmark_name: *const c_char)),
    pub(crate) clear_save_dir: vfn!((this: *mut c_void)),
    pub(crate) get_map_entities_string: vfn!((this: *mut c_void) -> *const c_char),
    pub(crate) text_message_get: vfn!((this: *mut c_void, name: *const c_char) -> *mut ClientTextMessage),
    pub(crate) log_print: vfn!((this: *mut c_void, msg: *const c_char)),
    pub(crate) is_log_enabled: vfn!((this: *mut c_void) -> bool),
    pub(crate) build_entity_cluster_list: vfn!((this: *mut c_void, edict: *mut Edict, pvs_info: *mut PVSInfoT)),
    pub(crate) solid_moved: vfn!((this: *mut c_void, solid_ent: *mut Edict, solid_collide: *mut ICollideable, prev_abs_origin: *const Vector, bounds_only: bool)),
    pub(crate) trigger_moved: vfn!((this: *mut c_void, trigger_ent: *mut Edict, bounds_only: bool)),
    pub(crate) create_spatial_partition: vfn!((this: *mut c_void, worldmin: *const Vector, worldmax: *const Vector) -> *mut ISpatialPartition),
    pub(crate) destroy_spatial_partition: vfn!((this: *mut c_void, partition: *mut ISpatialPartition)),
    pub(crate) draw_map_to_scratch_pad: vfn!((this: *mut c_void, pad: *mut IScratchPad3D, flags: u32)),
    pub(crate) get_entity_transmit_bits_for_client: vfn!((this: *mut c_void, client_index: c_int) -> *const CBitVec),
    pub(crate) is_paused: vfn!((this: *mut c_void) -> bool),
    pub(crate) get_timescale: vfn!((this: *mut c_void) -> f32),
    pub(crate) force_exact_file: vfn!((this: *mut c_void, s: *const c_char)),
    pub(crate) force_model_bounds: vfn!((this: *mut c_void, s: *const c_char, mins: *const Vector, maxs: *const Vector)),
    pub(crate) clear_save_dir_after_client_load: vfn!((this: *mut c_void)),
    pub(crate) set_fake_client_con_var_value: vfn!((this: *mut c_void, entity: *mut Edict, cvar: *const c_char, value: *const c_char)),
    pub(crate) force_simple_material: vfn!((this: *mut c_void, s: *const c_char)),
    pub(crate) is_in_commentary_mode: vfn!((this: *mut c_void) -> c_int),
    pub(crate) is_level_main_menu_background: vfn!((this: *mut c_void) -> bool),
    pub(crate) set_area_portal_states: vfn!((this: *mut c_void, portal_numbers: *const c_int, is_open: *const c_int, n_portals: c_int)),
    pub(crate) notify_edict_flags_change: vfn!((this: *mut c_void, edict_index: c_int)),
    pub(crate) get_prev_check_transmit_info: vfn!((this: *mut c_void, player_edict: *mut Edict) -> *const CCheckTransmitInfo),
    pub(crate) get_shared_edict_change_info: vfn!((this: *mut c_void) -> *mut CSharedEdictChangeInfo),
    pub(crate) allow_immediate_edict_reuse: vfn!((this: *mut c_void)),
    pub(crate) is_internal_build: vfn!((this: *mut c_void) -> bool),
    pub(crate) get_change_accessor: vfn!((this: *mut c_void, edict: *const Edict) -> *mut IChangeInfoAccessor),
    pub(crate) get_most_recently_loaded_file_name: vfn!((this: *mut c_void) -> *const c_char),
    pub(crate) get_save_file_name: vfn!((this: *mut c_void) -> *const c_char),
    pub(crate) clean_up_entity_cluster_list: vfn!((this: *mut c_void, pvs_info: *mut PVSInfoT)),
    pub(crate) get_app_id: vfn!((this: *mut c_void) -> c_int),
    pub(crate) is_low_violence: vfn!((this: *mut c_void) -> bool),
    pub(crate) is_any_client_low_violence: vfn!((this: *mut c_void) -> bool),
    pub(crate) start_query_cvar_value: vfn!((this: *mut c_void, player_entity: *mut Edict, name: *const c_char) -> QueryCvarCookieT),
    pub(crate) insert_server_command: vfn!((this: *mut c_void, str: *const c_char)),
    pub(crate) get_player_info: vfn!((this: *mut c_void, ent_num: c_int, pinfo: *mut PlayerInfo) -> bool),
    pub(crate) is_client_fully_authenticated: vfn!((this: *mut c_void, edict: *mut Edict) -> bool),
    pub(crate) set_dedicated_server_benchmark_mode: vfn!((this: *mut c_void, benchmark_mode: bool)),
    pub(crate) is_split_screen_player: vfn!((this: *mut c_void, ent_num: c_int) -> bool),
    pub(crate) get_split_screen_player_attach_to_edict: vfn!((this: *mut c_void, ent_num: c_int) -> *mut Edict),
    pub(crate) get_num_split_screen_users_attached_to_edict: vfn!((this: *mut c_void, ent_num: c_int) -> c_int),
    pub(crate) get_split_screen_player_for_edict: vfn!((this: *mut c_void, ent_num: c_int, slot: c_int) -> *mut Edict),
    pub(crate) is_override_load_game_ents_on: vfn!((this: *mut c_void) -> bool),
    pub(crate) force_flush_entity: vfn!((this: *mut c_void, entity_index: c_int)),
    pub(crate) get_single_player_shared_memory_space: vfn!((this: *mut c_void, name: *const c_char, ent_num: c_int) -> *mut ISPSharedMemory),
    pub(crate) alloc_level_static_data: vfn!((this: *mut c_void, bytes: usize) -> *mut c_void),
    pub(crate) get_cluster_count: vfn!((this: *mut c_void) -> c_int),
    pub(crate) get_all_cluster_bounds: vfn!((this: *mut c_void, bbox_list: *mut BBoxT, max_bbox: c_int) -> c_int),
    pub(crate) is_creating_reslist: vfn!((this: *mut c_void) -> bool),
    pub(crate) is_creating_xbox_reslist: vfn!((this: *mut c_void) -> bool),
    pub(crate) is_dedicated_server_for_xbox: vfn!((this: *mut c_void) -> bool),
    pub(crate) pause: vfn!((this: *mut c_void, pause: bool, force: bool)),
    pub(crate) set_timescale: vfn!((this: *mut c_void, timescale: f32)),
    pub(crate) set_gamestats_data: vfn!((this: *mut c_void, gamestats_data: *mut CGamestatsData)),
    pub(crate) get_gamestats_data: vfn!((this: *mut c_void) -> *mut CGamestatsData),
    pub(crate) get_client_steam_id: vfn!((this: *mut c_void, player_edict: *const Edict) -> *const CSteamID),
    pub(crate) get_game_server_steam_id: vfn!((this: *mut c_void) -> *const CSteamID),
    pub(crate) host_validate_session: vfn!((this: *mut c_void)),
    pub(crate) refresh_screen_if_necessary: vfn!((this: *mut c_void)),
    pub(crate) has_paintmap: vfn!((this: *mut c_void) -> bool),
    pub(crate) sphere_paint_surface: vfn!((this: *mut c_void, model: *const ModelT, pos: *const Vector, paint_type: u8, radius: f32, alpha: f32) -> bool),
    pub(crate) sphere_trace_paint_surface: vfn!((this: *mut c_void, model: *const ModelT, pos: *const Vector, dir: *const Vector, radius: f32, paint_types: *mut CUtlVector)),
    pub(crate) remove_all_paint: vfn!((this: *mut c_void)),
    pub(crate) paint_all_surfaces: vfn!((this: *mut c_void, paint_type: u8)),
    pub(crate) remove_paint: vfn!((this: *mut c_void, model: *const ModelT)),
    pub(crate) client_command_key_values: vfn!((this: *mut c_void, edict: *mut Edict, command: *mut KeyValues)),
    pub(crate) get_client_xuid: vfn!((this: *mut c_void, player_edict: *const Edict) -> u64),
    pub(crate) is_active_app: vfn!((this: *mut c_void) -> bool),
    pub(crate) set_no_clip_enabled: vfn!((this: *mut c_void, enabled: bool)),
    pub(crate) get_paintmap_data_rle: vfn!((this: *mut c_void, mapdata: *mut CUtlVector)),
    pub(crate) load_paintmap_data_rle: vfn!((this: *mut c_void, mapdata: *mut CUtlVector)),
    pub(crate) send_paintmap_data_to_client: vfn!((this: *mut c_void, edict: *mut Edict)),
    pub(crate) get_latency_for_choreo_sounds: vfn!((this: *mut c_void) -> f32),
    pub(crate) get_client_cross_play_platform: vfn!((this: *mut c_void, client_index: c_int) -> c_int),
}

// ==========================================================================
// Safe Rust Abstractions
// ==========================================================================

impl IVEngineServer {
    /// Tell engine to change level ( "changelevel s1\n" or "changelevel2 s1 s2\n" )
    pub fn change_level(&self, s1: &str, s2: &str) {
        if let (Ok(c1), Ok(c2)) = (CString::new(s1), CString::new(s2)) {
            unsafe { (self.change_level)(self.this, c1.as_ptr(), c2.as_ptr()) }
        }
    }

    /// Ask engine whether the specified map is a valid map file (exists and has valid version number).
    pub fn is_map_valid(&self, filename: &str) -> bool {
        if let Ok(c_name) = CString::new(filename) {
            unsafe { (self.is_map_valid)(self.this, c_name.as_ptr()) != 0 }
        } else { false }
    }

    /// Is this a dedicated server?
    pub fn is_dedicated_server(&self) -> bool {
        unsafe { (self.is_dedicated_server)(self.this) }
    }

    /// Is in Hammer editing mode?
    pub fn is_in_edit_mode(&self) -> bool {
        unsafe { (self.is_in_edit_mode)(self.this) != 0 }
    }

    /// Get arbitrary launch options. Returns a mutable reference to the root KeyValues node.
    pub fn get_launch_options<'a>(&self) -> Option<&'a mut KeyValues> {
        let ptr = unsafe { (self.get_launch_options)(self.this) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Add to the server lookup/precache table.
    pub fn precache_model(&self, name: &str, preload: bool) -> i32 {
        if let Ok(c_str) = CString::new(name) {
            unsafe { (self.precache_model)(self.this, c_str.as_ptr(), preload) }
        } else { 0 }
    }

    /// Precache a sentence file.
    pub fn precache_sentence_file(&self, name: &str, preload: bool) -> i32 {
        if let Ok(c_str) = CString::new(name) {
            unsafe { (self.precache_sentence_file)(self.this, c_str.as_ptr(), preload) }
        } else { 0 }
    }

    /// Precache a decal.
    pub fn precache_decal(&self, name: &str, preload: bool) -> i32 {
        if let Ok(c_str) = CString::new(name) {
            unsafe { (self.precache_decal)(self.this, c_str.as_ptr(), preload) }
        } else { 0 }
    }

    /// Precache a generic file.
    pub fn precache_generic(&self, name: &str, preload: bool) -> i32 {
        if let Ok(c_str) = CString::new(name) {
            unsafe { (self.precache_generic)(self.this, c_str.as_ptr(), preload) }
        } else { 0 }
    }

    /// Checks if the model name is precached.
    pub fn is_model_precached(&self, name: &str) -> bool {
        if let Ok(c_str) = CString::new(name) {
            unsafe { (self.is_model_precached)(self.this, c_str.as_ptr()) }
        } else { false }
    }

    /// Checks if the decal name is precached.
    pub fn is_decal_precached(&self, name: &str) -> bool {
        if let Ok(c_str) = CString::new(name) {
            unsafe { (self.is_decal_precached)(self.this, c_str.as_ptr()) }
        } else { false }
    }

    /// Checks if the generic file name is precached.
    pub fn is_generic_precached(&self, name: &str) -> bool {
        if let Ok(c_str) = CString::new(name) {
            unsafe { (self.is_generic_precached)(self.this, c_str.as_ptr()) }
        } else { false }
    }

    /// Get the cluster # for the specified position.
    pub fn get_cluster_for_origin(&self, org: &Vector) -> i32 {
        unsafe { (self.get_cluster_for_origin)(self.this, org) }
    }

    /// Get the PVS bits for a specified cluster. Returns the number of bytes needed to pack the PVS.
    pub fn get_pvs_for_cluster(&self, cluster: i32, output: &mut [u8]) -> i32 {
        unsafe { (self.get_pvs_for_cluster)(self.this, cluster, output.len() as c_int, output.as_mut_ptr()) }
    }

    /// Check whether the specified origin is inside the specified PVS.
    pub fn check_origin_in_pvs(&self, org: &Vector, check_pvs: &[u8]) -> bool {
        unsafe { (self.check_origin_in_pvs)(self.this, org, check_pvs.as_ptr(), check_pvs.len() as c_int) }
    }

    /// Check whether the specified worldspace bounding box is inside the specified PVS.
    pub fn check_box_in_pvs(&self, mins: &Vector, maxs: &Vector, check_pvs: &[u8]) -> bool {
        unsafe { (self.check_box_in_pvs)(self.this, mins, maxs, check_pvs.as_ptr(), check_pvs.len() as c_int) }
    }

    /// Returns the server assigned userid for this player. Returns -1 if not found.
    pub fn get_player_user_id(&self, edict: Option<&Edict>) -> i32 {
        let ptr = edict.map_or(ptr::null(), |e| e as *const _);
        unsafe { (self.get_player_user_id)(self.this, ptr) }
    }

    /// Returns the Network ID string (e.g. SteamID) for the player.
    pub fn get_player_network_id_string(&self, edict: Option<&Edict>) -> Option<String> {
        let ptr = edict.map_or(ptr::null(), |e| e as *const _);
        let c_str_ptr = unsafe { (self.get_player_network_id_string)(self.this, ptr) };
        if c_str_ptr.is_null() { None } else { Some(unsafe { CStr::from_ptr(c_str_ptr).to_string_lossy().into_owned() }) }
    }

    /// Returns true if the given UserID is currently in use.
    pub fn is_user_id_in_use(&self, user_id: i32) -> bool {
        unsafe { (self.is_user_id_in_use)(self.this, user_id) }
    }

    /// Gets the loading progress for a specific user ID.
    pub fn get_loading_progress_for_user_id(&self, user_id: i32) -> i32 {
        unsafe { (self.get_loading_progress_for_user_id)(self.this, user_id) }
    }

    /// Return the current number of used edict slots.
    pub fn get_entity_count(&self) -> i32 {
        unsafe { (self.get_entity_count)(self.this) }
    }

    /// Get stats info interface for a client netchannel.
    pub fn get_player_net_info<'a>(&self, player_index: i32) -> Option<&'a mut INetChannelInfo> {
        let ptr = unsafe { (self.get_player_net_info)(self.this, player_index) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Allocate space for a string and return an edict.
    pub fn create_edict<'a>(&self, force_edict_index: i32) -> Option<&'a mut Edict> {
        let ptr = unsafe { (self.create_edict)(self.this, force_edict_index) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Remove the specified edict and place it back into the free edict list.
    pub fn remove_edict(&self, edict: &mut Edict) {
        unsafe { (self.remove_edict)(self.this, edict as *mut _) }
    }

    /// Memory allocation for entity class data.
    pub fn pv_alloc_ent_private_data(&self, cb: i32) -> *mut c_void {
        unsafe { (self.pv_alloc_ent_private_data)(self.this, cb) }
    }

    /// Free entity private data memory.
    pub fn free_ent_private_data(&self, entity: *mut c_void) {
        unsafe { (self.free_ent_private_data)(self.this, entity) }
    }

    /// Save/restore memory allocator.
    pub fn save_alloc_memory(&self, num: usize, size: usize) -> *mut c_void {
        unsafe { (self.save_alloc_memory)(self.this, num, size) }
    }

    /// Save/restore memory freer.
    pub fn save_free_memory(&self, save_mem: *mut c_void) {
        unsafe { (self.save_free_memory)(self.this, save_mem) }
    }

    /// Emit an ambient sound associated with the specified entity.
    pub fn emit_ambient_sound(&self, entindex: i32, pos: &Vector, samp: &str, vol: f32, soundlevel: SoundLevelT, flags: i32, pitch: i32, delay: f32) {
        if let Ok(c_samp) = CString::new(samp) {
            unsafe { (self.emit_ambient_sound)(self.this, entindex, pos, c_samp.as_ptr(), vol, soundlevel, flags, pitch, delay) }
        }
    }

    /// Fade out the client's volume level toward silence.
    pub fn fade_client_volume(&self, edict: Option<&Edict>, fade_percent: f32, fade_out_seconds: f32, hold_time: f32, fade_in_seconds: f32) {
        let ptr = edict.map_or(ptr::null(), |e| e as *const _);
        unsafe { (self.fade_client_volume)(self.this, ptr, fade_percent, fade_out_seconds, hold_time, fade_in_seconds) }
    }

    /// Picks a sentence from a sentence group. Returns the sentence index.
    pub fn sentence_group_pick(&self, group_index: i32, name_buf: &mut [u8]) -> i32 {
        unsafe { (self.sentence_group_pick)(self.this, group_index, name_buf.as_mut_ptr() as *mut c_char, name_buf.len() as c_int) }
    }

    /// Sequential sentence picker.
    pub fn sentence_group_pick_sequential(&self, group_index: i32, name_buf: &mut [u8], sentence_index: i32, reset: i32) -> i32 {
        unsafe { (self.sentence_group_pick_sequential)(self.this, group_index, name_buf.as_mut_ptr() as *mut c_char, name_buf.len() as c_int, sentence_index, reset) }
    }

    /// Retrieves sentence index from name.
    pub fn sentence_index_from_name(&self, name: &str) -> i32 {
        if let Ok(c_name) = CString::new(name) {
            unsafe { (self.sentence_index_from_name)(self.this, c_name.as_ptr()) }
        } else { -1 }
    }

    /// Retrieves sentence name from index.
    pub fn sentence_name_from_index(&self, index: i32) -> Option<String> {
        let ptr = unsafe { (self.sentence_name_from_index)(self.this, index) };
        if ptr.is_null() { None } else { Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }) }
    }

    /// Retrieves group index from name.
    pub fn sentence_group_index_from_name(&self, name: &str) -> i32 {
        if let Ok(c_name) = CString::new(name) {
            unsafe { (self.sentence_group_index_from_name)(self.this, c_name.as_ptr()) }
        } else { -1 }
    }

    /// Retrieves group name from index.
    pub fn sentence_group_name_from_index(&self, index: i32) -> Option<String> {
        let ptr = unsafe { (self.sentence_group_name_from_index)(self.this, index) };
        if ptr.is_null() { None } else { Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }) }
    }

    /// Retrieves the length of a sentence.
    pub fn sentence_length(&self, index: i32) -> f32 {
        unsafe { (self.sentence_length)(self.this, index) }
    }

    /// Issue a command to the command parser as if it was typed at the server console.
    pub fn server_command(&self, cmd: &str) {
        if let Ok(c_str) = CString::new(cmd) {
            unsafe { (self.server_command)(self.this, c_str.as_ptr()) }
        }
    }

    /// Execute any commands currently in the command parser immediately.
    pub fn server_execute(&self) {
        unsafe { (self.server_execute)(self.this) }
    }

    /// Issue the specified command to the specified client (mimics client typing).
    /// Safely formats the command and avoids format string vulnerabilities.
    pub fn client_command(&self, edict: &mut Edict, cmd: &str) {
        let formatted = format!("{}\n\0", cmd); // Engine usually expects a newline for client cmds
        let fmt_specifier = b"%s\0".as_ptr() as *const c_char;
        unsafe {
            (self.client_command)(self.this, edict as *mut _, fmt_specifier, formatted.as_ptr() as *const c_char)
        }
    }

    /// Set the lightstyle to the specified value.
    pub fn light_style(&self, style: i32, val: &str) {
        if let Ok(c_val) = CString::new(val) {
            unsafe { (self.light_style)(self.this, style, c_val.as_ptr()) }
        }
    }

    /// Project a static decal onto the specified entity / model.
    pub fn static_decal(&self, origin: &Vector, decal_index: i32, entity_index: i32, model_index: i32, lowpriority: bool) {
        unsafe { (self.static_decal)(self.this, origin, decal_index, entity_index, model_index, lowpriority) }
    }

    /// Determine which players should hear/receive the multicast message.
    pub fn message_determine_multicast_recipients(&self, usepas: bool, origin: &Vector, playerbits: &mut CPlayerBitVec) {
        unsafe { (self.message_determine_multicast_recipients)(self.this, usepas, origin, playerbits as *mut _) }
    }

    /// Begin a message from a server side entity to its client side counterpart.
    pub fn entity_message_begin<'a>(&self, ent_index: i32, ent_class: &mut ServerClass, reliable: bool) -> Option<&'a mut BfWrite> {
        let ptr = unsafe { (self.entity_message_begin)(self.this, ent_index, ent_class as *mut _, reliable) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Begin a user message from the server to the client DLL.
    pub fn user_message_begin<'a>(&self, filter: &mut IRecipientFilter, msg_type: i32, msg_name: &str) -> Option<&'a mut BfWrite> {
        if let Ok(c_name) = CString::new(msg_name) {
            let ptr = unsafe { (self.user_message_begin)(self.this, filter as *mut _, msg_type, c_name.as_ptr()) };
            if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
        } else { None }
    }

    /// Finish the Entity or UserMessage and dispatch to network layer.
    pub fn message_end(&self) {
        unsafe { (self.message_end)(self.this) }
    }

    /// Print message to the client console.
    pub fn client_printf(&self, edict: &mut Edict, msg: &str) {
        if let Ok(c_msg) = CString::new(msg) {
            unsafe { (self.client_printf)(self.this, edict as *mut _, c_msg.as_ptr()) }
        }
    }

    /// Prints the string to the notification area of the screen (single player/listen server).
    pub fn con_nprintf(&self, pos: i32, msg: &str) {
        if let Ok(c_msg) = CString::new(msg) {
            let fmt_specifier = b"%s\0".as_ptr() as *const c_char;
            unsafe { (self.con_nprintf)(self.this, pos, fmt_specifier, c_msg.as_ptr()) }
        }
    }

    // Similar to `con_nprintf`, but allows specifying custom text color and duration.
    // pub fn con_nxprintf(&self, info: &ConNPrintS, msg: &str) {
    //     if let Ok(c_msg) = CString::new(msg) {
    //         let fmt_specifier = b"%s\0".as_ptr() as *const c_char;
    //         unsafe { (self.con_nxprintf)(self.this, info as *const _, fmt_specifier, c_msg.as_ptr()) }
    //     }
    // }

    /// Change a specified player's view entity.
    pub fn set_view(&self, client: Option<&Edict>, viewent: Option<&Edict>) {
        let c_ptr = client.map_or(ptr::null(), |e| e as *const _);
        let v_ptr = viewent.map_or(ptr::null(), |e| e as *const _);
        unsafe { (self.set_view)(self.this, c_ptr, v_ptr) }
    }

    /// Set the player's crosshair angle.
    pub fn crosshair_angle(&self, client: Option<&Edict>, pitch: f32, yaw: f32) {
        let ptr = client.map_or(ptr::null(), |e| e as *const _);
        unsafe { (self.crosshair_angle)(self.this, ptr, pitch, yaw) }
    }

    /// Get the current game directory (e.g., "hl2", "portal2").
    pub fn get_game_dir(&self) -> String {
        let mut buf = vec![0u8; 256];
        unsafe { (self.get_game_dir)(self.this, buf.as_mut_ptr() as *mut c_char, buf.len() as c_int) };
        unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char).to_string_lossy().into_owned() }
    }

    /// Compare two file times.
    pub fn compare_file_time(&self, filename1: &str, filename2: &str) -> Option<i32> {
        if let (Ok(c1), Ok(c2)) = (CString::new(filename1), CString::new(filename2)) {
            let mut compare = 0;
            let success = unsafe { (self.compare_file_time)(self.this, c1.as_ptr(), c2.as_ptr(), &mut compare) };
            if success != 0 { Some(compare) } else { None }
        } else { None }
    }

    /// Locks/unlocks the network string tables.
    pub fn lock_network_string_tables(&self, lock: bool) -> bool {
        unsafe { (self.lock_network_string_tables)(self.this, lock) }
    }

    /// Create a bot with the given name.
    pub fn create_fake_client<'a>(&self, netname: &str) -> Option<&'a mut Edict> {
        if let Ok(c_name) = CString::new(netname) {
            let ptr = unsafe { (self.create_fake_client)(self.this, c_name.as_ptr()) };
            if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
        } else { None }
    }

    /// Get a convar keyvalue for a specified client.
    pub fn get_client_con_var_value(&self, client_index: i32, name: &str) -> Option<String> {
        if let Ok(c_name) = CString::new(name) {
            let ptr = unsafe { (self.get_client_con_var_value)(self.this, client_index, c_name.as_ptr()) };
            if ptr.is_null() { None } else { Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }) }
        } else { None }
    }

    /// Parse a token from a file.
    pub fn parse_file(&self, data: &str, token_buf: &mut [u8]) -> Option<String> {
        if let Ok(c_data) = CString::new(data) {
            let ptr = unsafe { (self.parse_file)(self.this, c_data.as_ptr(), token_buf.as_mut_ptr() as *mut c_char, token_buf.len() as c_int) };
            if ptr.is_null() { None } else { Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }) }
        } else { None }
    }

    /// Copies a file.
    pub fn copy_file(&self, source: &str, dest: &str) -> bool {
        if let (Ok(c_src), Ok(c_dst)) = (CString::new(source), CString::new(dest)) {
            unsafe { (self.copy_file)(self.this, c_src.as_ptr(), c_dst.as_ptr()) }
        } else { false }
    }

    /// Reset the PVS buffer.
    pub fn reset_pvs(&self, pvs: &mut [u8]) {
        unsafe { (self.reset_pvs)(self.this, pvs.as_mut_ptr(), pvs.len() as c_int) }
    }

    /// Merge the PVS bits based on the specified origin.
    pub fn add_origin_to_pvs(&self, origin: &Vector) {
        unsafe { (self.add_origin_to_pvs)(self.this, origin) }
    }

    /// Mark an area portal as open/closed.
    pub fn set_area_portal_state(&self, portal_number: i32, is_open: bool) {
        unsafe { (self.set_area_portal_state)(self.this, portal_number, if is_open { 1 } else { 0 }) }
    }

    /// Queue a temp entity for transmission.
    pub fn playback_temp_entity(&self, filter: &mut IRecipientFilter, delay: f32, sender: *const c_void, st: &SendTable, class_id: i32) {
        unsafe { (self.playback_temp_entity)(self.this, filter as *mut _, delay, sender, st as *const _, class_id) }
    }

    /// Determine if the node is in the PVS.
    pub fn check_headnode_visible(&self, nodenum: i32, pvs: &[u8]) -> i32 {
        unsafe { (self.check_headnode_visible)(self.this, nodenum, pvs.as_ptr(), pvs.len() as c_int) }
    }

    /// Determine if area1 flows into area2.
    pub fn check_areas_connected(&self, area1: i32, area2: i32) -> i32 {
        unsafe { (self.check_areas_connected)(self.this, area1, area2) }
    }

    /// Get the area index for an origin.
    pub fn get_area(&self, origin: &Vector) -> i32 {
        unsafe { (self.get_area)(self.this, origin) }
    }

    /// Get area portal bit set.
    pub fn get_area_bits(&self, area: i32, bits: &mut [u8]) {
        unsafe { (self.get_area_bits)(self.this, area, bits.as_mut_ptr(), bits.len() as c_int) }
    }

    /// Get the plane leading out of an area through a portal.
    pub fn get_area_portal_plane(&self, view_origin: &Vector, portal_key: i32, plane: &mut VPlane) -> bool {
        unsafe { (self.get_area_portal_plane)(self.this, view_origin, portal_key, plane as *mut _) }
    }

    /// Load a saved game state.
    pub fn load_game_state(&self, map_name: &str, create_players: bool) -> bool {
        if let Ok(c_map) = CString::new(map_name) {
            unsafe { (self.load_game_state)(self.this, c_map.as_ptr(), create_players) }
        } else { false }
    }

    /// Load adjacent entities for transitions.
    pub fn load_adjacent_ents(&self, old_level: &str, landmark_name: &str) {
        if let (Ok(c_old), Ok(c_land)) = (CString::new(old_level), CString::new(landmark_name)) {
            unsafe { (self.load_adjacent_ents)(self.this, c_old.as_ptr(), c_land.as_ptr()) }
        }
    }

    /// Clears the save directory.
    pub fn clear_save_dir(&self) {
        unsafe { (self.clear_save_dir)(self.this) }
    }

    /// Get the pristine map entity lump string.
    pub fn get_map_entities_string(&self) -> Option<String> {
        let ptr = unsafe { (self.get_map_entities_string)(self.this) };
        if ptr.is_null() { None } else { Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }) }
    }

    /// Lookup a text message by name.
    pub fn text_message_get<'a>(&self, name: &str) -> Option<&'a mut ClientTextMessage> {
        if let Ok(c_name) = CString::new(name) {
            let ptr = unsafe { (self.text_message_get)(self.this, c_name.as_ptr()) };
            if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
        } else { None }
    }

    /// Print a message to the server log file.
    pub fn log_print(&self, msg: &str) {
        if let Ok(c_msg) = CString::new(msg) {
            unsafe { (self.log_print)(self.this, c_msg.as_ptr()) }
        }
    }

    /// Is logging enabled?
    pub fn is_log_enabled(&self) -> bool {
        unsafe { (self.is_log_enabled)(self.this) }
    }

    /// Build PVS cluster list for an entity.
    pub fn build_entity_cluster_list(&self, edict: &mut Edict, pvs_info: &mut PVSInfoT) {
        unsafe { (self.build_entity_cluster_list)(self.this, edict as *mut _, pvs_info as *mut _) }
    }

    /// A solid entity moved, update spatial partition.
    pub fn solid_moved(&self, solid_ent: &mut Edict, solid_collide: &mut ICollideable, prev_abs_origin: &Vector, bounds_only: bool) {
        unsafe { (self.solid_moved)(self.this, solid_ent as *mut _, solid_collide as *mut _, prev_abs_origin, bounds_only) }
    }

    /// A trigger entity moved, update spatial partition.
    pub fn trigger_moved(&self, trigger_ent: &mut Edict, bounds_only: bool) {
        unsafe { (self.trigger_moved)(self.this, trigger_ent as *mut _, bounds_only) }
    }

    /// Create a custom spatial partition.
    pub fn create_spatial_partition<'a>(&self, worldmin: &Vector, worldmax: &Vector) -> Option<&'a mut ISpatialPartition> {
        let ptr = unsafe { (self.create_spatial_partition)(self.this, worldmin, worldmax) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Destroy a custom spatial partition.
    pub fn destroy_spatial_partition(&self, partition: &mut ISpatialPartition) {
        unsafe { (self.destroy_spatial_partition)(self.this, partition as *mut _) }
    }

    /// Draw the brush geometry to the scratch pad.
    pub fn draw_map_to_scratch_pad(&self, pad: &mut IScratchPad3D, flags: u32) {
        unsafe { (self.draw_map_to_scratch_pad)(self.this, pad as *mut _, flags) }
    }

    /// Returns transmit bits indicating which entities the client knows about.
    pub fn get_entity_transmit_bits_for_client<'a>(&self, client_index: i32) -> Option<&'a CBitVec> {
        let ptr = unsafe { (self.get_entity_transmit_bits_for_client)(self.this, client_index) };
        if ptr.is_null() { None } else { Some(unsafe { &*ptr }) }
    }

    /// Is the game paused?
    pub fn is_paused(&self) -> bool {
        unsafe { (self.is_paused)(self.this) }
    }

    /// Returns the game timescale multiplied by the host timescale.
    pub fn get_timescale(&self) -> f32 {
        unsafe { (self.get_timescale)(self.this) }
    }

    /// Force exact file consistency checking.
    pub fn force_exact_file(&self, filename: &str) {
        if let Ok(c_str) = CString::new(filename) {
            unsafe { (self.force_exact_file)(self.this, c_str.as_ptr()) }
        }
    }

    /// Force model bounds consistency checking.
    pub fn force_model_bounds(&self, filename: &str, mins: &Vector, maxs: &Vector) {
        if let Ok(c_str) = CString::new(filename) {
            unsafe { (self.force_model_bounds)(self.this, c_str.as_ptr(), mins, maxs) }
        }
    }

    /// Clear save dir after client loads.
    pub fn clear_save_dir_after_client_load(&self) {
        unsafe { (self.clear_save_dir_after_client_load)(self.this) }
    }

    /// Sets a USERINFO client ConVar for a fake client.
    pub fn set_fake_client_con_var_value(&self, entity: &mut Edict, cvar: &str, value: &str) {
        if let (Ok(c_cvar), Ok(c_val)) = (CString::new(cvar), CString::new(value)) {
            unsafe { (self.set_fake_client_con_var_value)(self.this, entity as *mut _, c_cvar.as_ptr(), c_val.as_ptr()) }
        }
    }

    /// Force simple material consistency checking.
    pub fn force_simple_material(&self, material: &str) {
        if let Ok(c_str) = CString::new(material) {
            unsafe { (self.force_simple_material)(self.this, c_str.as_ptr()) }
        }
    }

    /// Is the engine in Commentary mode?
    pub fn is_in_commentary_mode(&self) -> bool {
        unsafe { (self.is_in_commentary_mode)(self.this) != 0 }
    }

    /// Is the engine running a background map?
    pub fn is_level_main_menu_background(&self) -> bool {
        unsafe { (self.is_level_main_menu_background)(self.this) }
    }

    /// Mark an array of area portals as open/closed.
    pub fn set_area_portal_states(&self, portal_numbers: &[i32], is_open: &[i32]) {
        let count = std::cmp::min(portal_numbers.len(), is_open.len()) as c_int;
        unsafe { (self.set_area_portal_states)(self.this, portal_numbers.as_ptr(), is_open.as_ptr(), count) }
    }

    /// Called when relevant edict state flags change.
    pub fn notify_edict_flags_change(&self, edict_index: i32) {
        unsafe { (self.notify_edict_flags_change)(self.this, edict_index) }
    }

    /// Retrieve previous transmit check info.
    pub fn get_prev_check_transmit_info<'a>(&self, player_edict: &mut Edict) -> Option<&'a CCheckTransmitInfo> {
        let ptr = unsafe { (self.get_prev_check_transmit_info)(self.this, player_edict as *mut _) };
        if ptr.is_null() { None } else { Some(unsafe { &*ptr }) }
    }

    /// Retrieve shared edict change info.
    pub fn get_shared_edict_change_info<'a>(&self) -> Option<&'a mut CSharedEdictChangeInfo> {
        let ptr = unsafe { (self.get_shared_edict_change_info)(self.this) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Allow immediate edict reuse.
    pub fn allow_immediate_edict_reuse(&self) {
        unsafe { (self.allow_immediate_edict_reuse)(self.this) }
    }

    /// Is the engine an internal build?
    pub fn is_internal_build(&self) -> bool {
        unsafe { (self.is_internal_build)(self.this) }
    }

    /// Get change accessor for an edict.
    pub fn get_change_accessor<'a>(&self, edict: &Edict) -> Option<&'a mut IChangeInfoAccessor> {
        let ptr = unsafe { (self.get_change_accessor)(self.this, edict as *const _) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Name of most recently load .sav file.
    pub fn get_most_recently_loaded_file_name(&self) -> Option<String> {
        let ptr = unsafe { (self.get_most_recently_loaded_file_name)(self.this) };
        if ptr.is_null() { None } else { Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }) }
    }

    /// Get save file name.
    pub fn get_save_file_name(&self) -> Option<String> {
        let ptr = unsafe { (self.get_save_file_name)(self.this) };
        if ptr.is_null() { None } else { Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }) }
    }

    /// Cleans up the cluster list.
    pub fn clean_up_entity_cluster_list(&self, pvs_info: &mut PVSInfoT) {
        unsafe { (self.clean_up_entity_cluster_list)(self.this, pvs_info as *mut _) }
    }

    /// Gets the Steam AppID of the current game.
    pub fn get_app_id(&self) -> i32 {
        unsafe { (self.get_app_id)(self.this) }
    }

    /// Returns true if the game is running in low violence mode.
    pub fn is_low_violence(&self) -> bool {
        unsafe { (self.is_low_violence)(self.this) }
    }

    /// Returns true if any connected client is running in low violence mode.
    pub fn is_any_client_low_violence(&self) -> bool {
        unsafe { (self.is_any_client_low_violence)(self.this) }
    }

    /// Starts an async query to get a client's cvar value.
    pub fn start_query_cvar_value(&self, player_entity: &mut Edict, name: &str) -> QueryCvarCookieT {
        if let Ok(c_name) = CString::new(name) {
            unsafe { (self.start_query_cvar_value)(self.this, player_entity as *mut _, c_name.as_ptr()) }
        } else { -1 }
    }

    /// Insert a server command into the buffer.
    pub fn insert_server_command(&self, cmd: &str) {
        if let Ok(c_str) = CString::new(cmd) {
            unsafe { (self.insert_server_command)(self.this, c_str.as_ptr()) }
        }
    }

    /// Populates PlayerInfo for a specific client index (starts at 1).
    pub fn get_player_info(&self, ent_num: i32) -> Option<PlayerInfo> {
        let mut info = PlayerInfo::default();
        if unsafe { (self.get_player_info)(self.this, ent_num, &mut info) } { Some(info) } else { None }
    }

    /// Returns true if this client has been fully authenticated by Steam.
    pub fn is_client_fully_authenticated(&self, edict: &mut Edict) -> bool {
        unsafe { (self.is_client_fully_authenticated)(self.this, edict as *mut _) }
    }

    /// Sets whether the server runs in benchmark mode (1 tick per frame).
    pub fn set_dedicated_server_benchmark_mode(&self, benchmark_mode: bool) {
        unsafe { (self.set_dedicated_server_benchmark_mode)(self.this, benchmark_mode) }
    }

    /// Returns true if the player is a split screen player.
    pub fn is_split_screen_player(&self, ent_num: i32) -> bool {
        unsafe { (self.is_split_screen_player)(self.this, ent_num) }
    }

    /// Gets the edict the splitscreen player is attached to.
    pub fn get_split_screen_player_attach_to_edict<'a>(&self, ent_num: i32) -> Option<&'a mut Edict> {
        let ptr = unsafe { (self.get_split_screen_player_attach_to_edict)(self.this, ent_num) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Gets the number of splitscreen users attached to an edict.
    pub fn get_num_split_screen_users_attached_to_edict(&self, ent_num: i32) -> i32 {
        unsafe { (self.get_num_split_screen_users_attached_to_edict)(self.this, ent_num) }
    }

    /// Gets the splitscreen player edict by slot.
    pub fn get_split_screen_player_for_edict<'a>(&self, ent_num: i32, slot: i32) -> Option<&'a mut Edict> {
        let ptr = unsafe { (self.get_split_screen_player_for_edict)(self.this, ent_num, slot) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Is Foundry override load game entities active?
    pub fn is_override_load_game_ents_on(&self) -> bool {
        unsafe { (self.is_override_load_game_ents_on)(self.this) }
    }

    /// Force flush an entity for Foundry.
    pub fn force_flush_entity(&self, entity_index: i32) {
        unsafe { (self.force_flush_entity)(self.this, entity_index) }
    }

    /// Gets a single player shared memory space.
    pub fn get_single_player_shared_memory_space<'a>(&self, name: &str, ent_num: i32) -> Option<&'a mut ISPSharedMemory> {
        if let Ok(c_name) = CString::new(name) {
            let ptr = unsafe { (self.get_single_player_shared_memory_space)(self.this, c_name.as_ptr(), ent_num) };
            if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
        } else { None }
    }

    /// Allocate hunk memory.
    pub fn alloc_level_static_data(&self, bytes: usize) -> *mut c_void {
        unsafe { (self.alloc_level_static_data)(self.this, bytes) }
    }

    /// Gets the total number of clusters.
    pub fn get_cluster_count(&self) -> i32 {
        unsafe { (self.get_cluster_count)(self.this) }
    }

    /// Gets bounding boxes for all clusters.
    pub fn get_all_cluster_bounds(&self, bbox_list: &mut [BBoxT]) -> i32 {
        unsafe { (self.get_all_cluster_bounds)(self.this, bbox_list.as_mut_ptr(), bbox_list.len() as c_int) }
    }

    /// Is creating reslist?
    pub fn is_creating_reslist(&self) -> bool { unsafe { (self.is_creating_reslist)(self.this) } }
    pub fn is_creating_xbox_reslist(&self) -> bool { unsafe { (self.is_creating_xbox_reslist)(self.this) } }
    pub fn is_dedicated_server_for_xbox(&self) -> bool { unsafe { (self.is_dedicated_server_for_xbox)(self.this) } }

    /// Pauses or unpauses the game.
    pub fn pause(&self, pause: bool, force: bool) {
        unsafe { (self.pause)(self.this, pause, force) }
    }

    /// Sets the timescale.
    pub fn set_timescale(&self, timescale: f32) {
        unsafe { (self.set_timescale)(self.this, timescale) }
    }

    /// Sets the gamestats data.
    pub fn set_gamestats_data(&self, gamestats_data: &mut CGamestatsData) {
        unsafe { (self.set_gamestats_data)(self.this, gamestats_data as *mut _) }
    }

    /// Gets the gamestats data.
    pub fn get_gamestats_data<'a>(&self) -> Option<&'a mut CGamestatsData> {
        let ptr = unsafe { (self.get_gamestats_data)(self.this) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Retrieves the SteamID of a connected client.
    pub fn get_client_steam_id(&self, player_edict: &Edict) -> Option<CSteamID> {
        let ptr = unsafe { (self.get_client_steam_id)(self.this, player_edict as *const _) };
        if ptr.is_null() { None } else { Some(unsafe { *ptr }) }
    }

    /// Retrieves the SteamID of the Game Server itself.
    pub fn get_game_server_steam_id(&self) -> Option<CSteamID> {
        let ptr = unsafe { (self.get_game_server_steam_id)(self.this) };
        if ptr.is_null() { None } else { Some(unsafe { *ptr }) }
    }

    /// Validates the current matchmaking session.
    pub fn host_validate_session(&self) {
        unsafe { (self.host_validate_session)(self.this) }
    }

    /// Refreshes the screen if necessary (primarily for Xbox loading screens).
    pub fn refresh_screen_if_necessary(&self) {
        unsafe { (self.refresh_screen_if_necessary)(self.this) }
    }

    /// Checks if the map has a paintmap (used in Portal 2).
    pub fn has_paintmap(&self) -> bool {
        unsafe { (self.has_paintmap)(self.this) }
    }

    /// Shoots a paint sphere onto surfaces.
    pub fn sphere_paint_surface(&self, model: &ModelT, pos: &Vector, paint_type: u8, radius: f32, alpha: f32) -> bool {
        unsafe { (self.sphere_paint_surface)(self.this, model as *const _, pos, paint_type, radius, alpha) }
    }

    /// Traces a paint sphere and collects affected paint types.
    pub fn sphere_trace_paint_surface(&self, model: &ModelT, pos: &Vector, dir: &Vector, radius: f32, paint_types: &mut CUtlVector) {
        unsafe { (self.sphere_trace_paint_surface)(self.this, model as *const _, pos, dir, radius, paint_types as *mut _) }
    }

    /// Removes all paint from the level.
    pub fn remove_all_paint(&self) {
        unsafe { (self.remove_all_paint)(self.this) }
    }

    /// Paints all surfaces in the level with a specific paint type.
    pub fn paint_all_surfaces(&self, paint_type: u8) {
        unsafe { (self.paint_all_surfaces)(self.this, paint_type) }
    }

    /// Removes paint from a specific model.
    pub fn remove_paint(&self, model: &ModelT) {
        unsafe { (self.remove_paint)(self.this, model as *const _) }
    }

    /// Sends a client command using KeyValues.
    pub fn client_command_key_values(&self, edict: &mut Edict, command: &mut KeyValues) {
        unsafe { (self.client_command_key_values)(self.this, edict as *mut _, command as *mut _) }
    }

    /// Gets the Xbox User ID (XUID) for a player.
    pub fn get_client_xuid(&self, player_edict: &Edict) -> u64 {
        unsafe { (self.get_client_xuid)(self.this, player_edict as *const _) }
    }

    /// Is this the active application?
    pub fn is_active_app(&self) -> bool {
        unsafe { (self.is_active_app)(self.this) }
    }

    /// Toggles the noclip mode globally.
    pub fn set_no_clip_enabled(&self, enabled: bool) {
        unsafe { (self.set_no_clip_enabled)(self.this, enabled) }
    }

    /// Retrieves compressed RLE data for the paintmap.
    pub fn get_paintmap_data_rle(&self, mapdata: &mut CUtlVector) {
        unsafe { (self.get_paintmap_data_rle)(self.this, mapdata as *mut _) }
    }

    /// Loads compressed RLE data to restore paint state.
    pub fn load_paintmap_data_rle(&self, mapdata: &mut CUtlVector) {
        unsafe { (self.load_paintmap_data_rle)(self.this, mapdata as *mut _) }
    }

    /// Synchronizes paintmap data with a specific client.
    pub fn send_paintmap_data_to_client(&self, edict: &mut Edict) {
        unsafe { (self.send_paintmap_data_to_client)(self.this, edict as *mut _) }
    }

    /// Gets latency specifically for choreo sound sync.
    pub fn get_latency_for_choreo_sounds(&self) -> f32 {
        unsafe { (self.get_latency_for_choreo_sounds)(self.this) }
    }

    /// Gets the crossplay platform indicator for the client.
    pub fn get_client_cross_play_platform(&self, client_index: i32) -> i32 {
        unsafe { (self.get_client_cross_play_platform)(self.this, client_index) }
    }
}
