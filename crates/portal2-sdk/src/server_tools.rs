#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;

use crate::types::{CBaseEntity, CEntityRespawnInfo, IClientEntity, IServerEntity, QAngle, Vector};
use crate::platform::abi::vfn;

// ==========================================================================
// Raw FFI Function Pointer Types
// ==========================================================================

type FnGetIServerEntity = vfn!((this: *mut c_void, p_client_entity: *const IClientEntity) -> *mut IServerEntity);
type FnSnapPlayerToPosition = vfn!((this: *mut c_void, org: *const Vector, ang: *const QAngle, p_client_player: *const IClientEntity) -> bool);
type FnGetPlayerPosition = vfn!((this: *mut c_void, org: *mut Vector, ang: *mut QAngle, p_client_player: *const IClientEntity) -> bool);
type FnSetPlayerFOV = vfn!((this: *mut c_void, fov: c_int, p_client_player: *const IClientEntity) -> bool);
type FnGetPlayerFOV = vfn!((this: *mut c_void, p_client_player: *const IClientEntity) -> c_int);
type FnIsInNoClipMode = vfn!((this: *mut c_void, p_client_player: *const IClientEntity) -> bool);

type FnFirstEntity = vfn!((this: *mut c_void) -> *mut CBaseEntity);
type FnNextEntity = vfn!((this: *mut c_void, p_entity: *const CBaseEntity) -> *mut CBaseEntity);
type FnFindEntityByHammerID = vfn!((this: *mut c_void, i_hammer_id: c_int) -> *mut CBaseEntity);

type FnGetKeyValue = vfn!((this: *mut c_void, p_entity: *const CBaseEntity, sz_field: *const c_char, sz_value: *mut c_char, i_max_len: c_int) -> bool);
type FnSetKeyValueStr = vfn!((this: *mut c_void, p_entity: *mut CBaseEntity, sz_field: *const c_char, sz_value: *const c_char) -> bool);
type FnSetKeyValueFlt = vfn!((this: *mut c_void, p_entity: *mut CBaseEntity, sz_field: *const c_char, fl_value: f32) -> bool);
type FnSetKeyValueVec = vfn!((this: *mut c_void, p_entity: *mut CBaseEntity, sz_field: *const c_char, vec_value: *const Vector) -> bool);

type FnCreateEntityByName = vfn!((this: *mut c_void, sz_class_name: *const c_char) -> *mut CBaseEntity);
type FnDispatchSpawn = vfn!((this: *mut c_void, p_entity: *mut CBaseEntity));
type FnDestroyEntityByHammerId = vfn!((this: *mut c_void, i_hammer_id: c_int) -> bool);
type FnRespawnEntitiesWithEdits = vfn!((this: *mut c_void, p_infos: *mut CEntityRespawnInfo, n_infos: c_int) -> bool);

type FnReloadParticleDefintions = vfn!((this: *mut c_void, p_file_name: *const c_char, p_buf_data: *const c_void, n_len: c_int));
type FnAddOriginToPVS = vfn!((this: *mut c_void, org: *const Vector));
type FnMoveEngineViewTo = vfn!((this: *mut c_void, v_pos: *const Vector, v_angles: *const QAngle));
type FnRemoveEntity = vfn!((this: *mut c_void, n_hammer_id: c_int));

// ==========================================================================
// Interface Structure
// ==========================================================================

/// Interface from the engine to tools for manipulating entities.
/// Provides safe abstractions to access underlying engine features.
pub struct IServerTools {
    pub(crate) this: *mut c_void,

    // VTable index 0-5
    pub(crate) get_iserver_entity: FnGetIServerEntity,
    pub(crate) snap_player_to_position: FnSnapPlayerToPosition,
    pub(crate) get_player_position: FnGetPlayerPosition,
    pub(crate) set_player_fov: FnSetPlayerFOV,
    pub(crate) get_player_fov: FnGetPlayerFOV,
    pub(crate) is_in_no_clip_mode: FnIsInNoClipMode,

    // VTable index 6-8
    pub(crate) first_entity: FnFirstEntity,
    pub(crate) next_entity: FnNextEntity,
    pub(crate) find_entity_by_hammer_id: FnFindEntityByHammerID,

    // VTable index 9-12
    pub(crate) get_key_value: FnGetKeyValue,
    pub(crate) set_key_value_str: FnSetKeyValueStr,
    pub(crate) set_key_value_flt: FnSetKeyValueFlt,
    pub(crate) set_key_value_vec: FnSetKeyValueVec,

    // VTable index 13-16
    pub(crate) create_entity_by_name: FnCreateEntityByName,
    pub(crate) dispatch_spawn: FnDispatchSpawn,
    pub(crate) destroy_entity_by_hammer_id: FnDestroyEntityByHammerId,
    pub(crate) respawn_entities_with_edits: FnRespawnEntitiesWithEdits,

    // VTable index 17-20
    pub(crate) reload_particle_defintions: FnReloadParticleDefintions,
    pub(crate) add_origin_to_pvs: FnAddOriginToPVS,
    pub(crate) move_engine_view_to: FnMoveEngineViewTo,
    pub(crate) remove_entity: FnRemoveEntity,
}

// ==========================================================================
// Safe Rust Abstractions
// ==========================================================================

impl IServerTools {
    /// Retrieves the server-side entity interface associated with a given client entity.
    pub fn get_server_entity<'a>(&self, client_entity: &IClientEntity) -> Option<&'a mut IServerEntity> {
        let ptr = unsafe { (self.get_iserver_entity)(self.this, client_entity as *const _) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Snaps a player's position and view angles directly to the specified values.
    /// If `client_player` is `None`, the engine usually targets the local/primary player.
    pub fn snap_player_to_position(&self, org: &Vector, ang: &QAngle, client_player: Option<&IClientEntity>) -> bool {
        let player_ptr = client_player.map_or(ptr::null(), |p| p as *const _);
        unsafe { (self.snap_player_to_position)(self.this, org, ang, player_ptr) }
    }

    /// Retrieves the current position and view angles of a player.
    /// Returns `Some((Vector, QAngle))` on success, or `None` if it failed.
    pub fn get_player_position(&self, client_player: Option<&IClientEntity>) -> Option<(Vector, QAngle)> {
        let player_ptr = client_player.map_or(ptr::null(), |p| p as *const _);
        let mut org = Vector::default();
        let mut ang = QAngle::default();

        let success = unsafe { (self.get_player_position)(self.this, &mut org, &mut ang, player_ptr) };
        if success { Some((org, ang)) } else { None }
    }

    /// Sets the Field of View (FOV) for a player.
    pub fn set_player_fov(&self, fov: i32, client_player: Option<&IClientEntity>) -> bool {
        let player_ptr = client_player.map_or(ptr::null(), |p| p as *const _);
        unsafe { (self.set_player_fov)(self.this, fov as c_int, player_ptr) }
    }

    /// Gets the current Field of View (FOV) of a player.
    pub fn get_player_fov(&self, client_player: Option<&IClientEntity>) -> i32 {
        let player_ptr = client_player.map_or(ptr::null(), |p| p as *const _);
        unsafe { (self.get_player_fov)(self.this, player_ptr) as i32 }
    }

    /// Checks if a player is currently in noclip mode.
    pub fn is_in_no_clip_mode(&self, client_player: Option<&IClientEntity>) -> bool {
        let player_ptr = client_player.map_or(ptr::null(), |p| p as *const _);
        unsafe { (self.is_in_no_clip_mode)(self.this, player_ptr) }
    }

    /// Returns a mutable reference to the first entity in the global entity list.
    /// Use `next_entity` to iterate over the list.
    pub fn first_entity<'a>(&self) -> Option<&'a mut CBaseEntity> {
        let ptr = unsafe { (self.first_entity)(self.this) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Returns a mutable reference to the next entity following the provided one.
    pub fn next_entity<'a>(&self, entity: &CBaseEntity) -> Option<&'a mut CBaseEntity> {
        let ptr = unsafe { (self.next_entity)(self.this, entity as *const _) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Finds an entity by its Hammer ID (the ID assigned to it in the map editor).
    pub fn find_entity_by_hammer_id<'a>(&self, hammer_id: i32) -> Option<&'a mut CBaseEntity> {
        let ptr = unsafe { (self.find_entity_by_hammer_id)(self.this, hammer_id as c_int) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Retrieves a KeyValue property from an entity as a String.
    /// Returns `None` if the field doesn't exist or string parsing failed.
    pub fn get_key_value(&self, entity: &CBaseEntity, field: &str) -> Option<String> {
        let c_field = CString::new(field).ok()?;
        let mut buf = vec![0u8; 1024]; // Standard safe buffer size for Source key values

        let success = unsafe {
            (self.get_key_value)(self.this, entity as *const _, c_field.as_ptr(), buf.as_mut_ptr() as *mut c_char, buf.len() as c_int)
        };

        if success {
            unsafe {
                let c_str = CStr::from_ptr(buf.as_ptr() as *const c_char);
                Some(c_str.to_string_lossy().into_owned())
            }
        } else {
            None
        }
    }

    /// Sets a KeyValue string property on an entity.
    /// Returns `false` if string allocation fails or engine rejects the change.
    pub fn set_key_value_str(&self, entity: &mut CBaseEntity, field: &str, value: &str) -> bool {
        let c_field = match CString::new(field) { Ok(s) => s, Err(_) => return false };
        let c_value = match CString::new(value) { Ok(s) => s, Err(_) => return false };
        unsafe { (self.set_key_value_str)(self.this, entity as *mut _, c_field.as_ptr(), c_value.as_ptr()) }
    }

    /// Sets a KeyValue float property on an entity.
    pub fn set_key_value_flt(&self, entity: &mut CBaseEntity, field: &str, value: f32) -> bool {
        let c_field = match CString::new(field) { Ok(s) => s, Err(_) => return false };
        unsafe { (self.set_key_value_flt)(self.this, entity as *mut _, c_field.as_ptr(), value) }
    }

    /// Sets a KeyValue vector property on an entity.
    pub fn set_key_value_vec(&self, entity: &mut CBaseEntity, field: &str, value: &Vector) -> bool {
        let c_field = match CString::new(field) { Ok(s) => s, Err(_) => return false };
        unsafe { (self.set_key_value_vec)(self.this, entity as *mut _, c_field.as_ptr(), value as *const _) }
    }

    /// Creates an entity by its class name (e.g. "prop_dynamic").
    /// The entity is created but not spawned. You must call `dispatch_spawn` to fully initialize it.
    pub fn create_entity_by_name<'a>(&self, classname: &str) -> Option<&'a mut CBaseEntity> {
        let c_str = CString::new(classname).ok()?;
        let ptr = unsafe { (self.create_entity_by_name)(self.this, c_str.as_ptr()) };
        if ptr.is_null() { None } else { Some(unsafe { &mut *ptr }) }
    }

    /// Fully spawns an entity into the game world.
    /// This should be called after `create_entity_by_name` and setting any necessary KeyValues.
    pub fn dispatch_spawn(&self, entity: &mut CBaseEntity) {
        unsafe { (self.dispatch_spawn)(self.this, entity as *mut _) }
    }

    /// Destroys an entity matching the given Hammer ID.
    pub fn destroy_entity_by_hammer_id(&self, hammer_id: i32) -> bool {
        unsafe { (self.destroy_entity_by_hammer_id)(self.this, hammer_id as c_int) }
    }

    /// Respawns a batch of entities with edits. It maintains the EHANDLEs validity
    /// so references to the entities remain intact.
    pub fn respawn_entities_with_edits(&self, infos: &mut [CEntityRespawnInfo]) -> bool {
        unsafe {
            (self.respawn_entities_with_edits)(self.this, infos.as_mut_ptr(), infos.len() as c_int)
        }
    }

    /// Reloads a portion or all of a particle definition file.
    /// The `data` slice contains the raw contents of the PCF file.
    pub fn reload_particle_definitions(&self, filename: &str, data: &[u8]) {
        if let Ok(c_filename) = CString::new(filename) {
            unsafe {
                (self.reload_particle_defintions)(
                    self.this,
                    c_filename.as_ptr(),
                    data.as_ptr() as *const c_void,
                    data.len() as c_int
                );
            }
        }
    }

    /// Manually adds a spatial origin point to the engine's PVS (Potentially Visible Set) calculation.
    pub fn add_origin_to_pvs(&self, org: &Vector) {
        unsafe { (self.add_origin_to_pvs)(self.this, org as *const _) }
    }

    /// Forcibly moves the engine's rendering view to a specified position and orientation.
    pub fn move_engine_view_to(&self, pos: &Vector, angles: &QAngle) {
        unsafe { (self.move_engine_view_to)(self.this, pos as *const _, angles as *const _) }
    }

    /// Calls `UTIL_Remove` on the entity corresponding to the given Hammer ID.
    pub fn remove_entity(&self, hammer_id: i32) {
        unsafe { (self.remove_entity)(self.this, hammer_id as c_int) }
    }
}
