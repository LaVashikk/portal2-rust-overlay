use std::ffi::{CStr, c_char, c_int};
use super::{Vector, QAngle, SendTable};

/// A unique identifier for a networkable entity. It combines an entity index
/// with a serial number to prevent stale handles from referring to new entities
/// that have taken the same index.
#[repr(transparent)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CBaseHandle(pub u32);

/// Describes the network class of an entity.
#[repr(C)]
pub struct ServerClass {
    pub name: *const c_char,
    pub table: *mut SendTable,
    pub next: *mut ServerClass,
    pub class_id: c_int,
    pub instance_baseline_index: c_int,
}

impl ServerClass {
    /// Returns the network name of the class (e.g., "CPropPhysics", "CTerrorPlayer").
    pub fn get_name(&self) -> String {
        unsafe { CStr::from_ptr(self.name).to_string_lossy().into_owned() }
    }
}

// ==========================================================================
// IServerNetworkable
// ==========================================================================
#[repr(C)] pub struct IServerNetworkable { _private: [u8; 0] }

// uuuuuuuuuuuugh... im so lazy. so sorry
impl IServerNetworkable {
    // VTable index 0: GetEntityHandle
    // VTable index 1: GetServerClass
    // VTable index 2: GetEdict
    // VTable index 3: GetClassName
    // VTable index 4: Release
    // VTable index 5: AreaNum
    // VTable index 6: GetBaseNetworkable
    // VTable index 7: GetBaseEntity
    // VTable index 8: GetPVSInfo
    // VTable index 9: ~IServerNetworkable

    /// Returns the ServerClass associated with this entity.
    pub fn get_server_class<'a>(&self) -> Option<&'a mut ServerClass> {
        unsafe {
            let vtable = *(self as *const _ as *const *const usize);
            let get_class: unsafe extern "thiscall" fn(*const IServerNetworkable) -> *mut ServerClass = std::mem::transmute(vtable.add(1).read());
            let ptr = get_class(self);
            if ptr.is_null() { None } else { Some(&mut *ptr) }
        }
    }

    /// Returns the network slot (Edict) attached to this entity.
    pub fn get_edict<'a>(&self) -> Option<&'a mut Edict> {
        unsafe {
            let vtable = *(self as *const _ as *const *const usize);
            let get_edict: unsafe extern "thiscall" fn(*const IServerNetworkable) -> *mut Edict = std::mem::transmute(vtable.add(2).read());
            let ptr = get_edict(self);
            if ptr.is_null() { None } else { Some(&mut *ptr) }
        }
    }

    /// Returns the hardcoded C++ class name (e.g., "prop_dynamic").
    pub fn get_class_name(&self) -> String {
        unsafe {
            let vtable = *(self as *const _ as *const *const usize);
            let get_name: unsafe extern "thiscall" fn(*const IServerNetworkable) -> *const c_char = std::mem::transmute(vtable.add(3).read());
            let ptr = get_name(self);
            if ptr.is_null() { String::new() } else { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
        }
    }
}


#[repr(C)] pub struct IServerEntity { _private: [u8; 0] }

impl IServerEntity {
    // VTable structure based on inheritance:
    // IHandleEntity:
    //   0: ~IHandleEntity
    //   1: SetRefEHandle
    //   2: GetRefEHandle
    // IServerUnknown:
    //   3: GetCollideable
    //   4: GetNetworkable
    //   5: GetBaseEntity
    // IServerEntity:
    //   0: ~IServerEntity (Overrides destructor, takes slot 0)
    //   6: GetModelIndex
    //   7: GetModelName
    //   8: SetModelIndex

    /// Returns the collision interface for this entity (bounding boxes, raycasts).
    pub fn get_collideable<'a>(&self) -> Option<&'a mut ICollideable> {
        unsafe {
            let vtable = *(self as *const _ as *const *const usize);
            let get_col: unsafe extern "thiscall" fn(*const IServerEntity) -> *mut ICollideable = std::mem::transmute(vtable.add(3).read());
            let ptr = get_col(self);
            if ptr.is_null() { None } else { Some(&mut *ptr) }
        }
    }

    /// Returns the networkable interface (allows getting Edicts and ServerClasses).
    pub fn get_networkable<'a>(&self) -> Option<&'a mut IServerNetworkable> {
        unsafe {
            let vtable = *(self as *const _ as *const *const usize);
            let get_net: unsafe extern "thiscall" fn(*const IServerEntity) -> *mut IServerNetworkable = std::mem::transmute(vtable.add(4).read());
            let ptr = get_net(self);
            if ptr.is_null() { None } else { Some(&mut *ptr) }
        }
    }

    /// Returns the entity handle, which contains the entity index.
    pub fn get_handle(&self) -> CBaseHandle {
        unsafe {
            let vtable = *(self as *const _ as *const *const usize);
            let get_handle: unsafe extern "thiscall" fn(*const IServerEntity) -> CBaseHandle = std::mem::transmute(vtable.add(2).read());
            get_handle(self)
        }
    }

    /// Returns the model index of this entity.
    pub fn get_model_index(&self) -> i32 {
        unsafe {
            let vtable = *(self as *const _ as *const *const usize);
            let get_idx: unsafe extern "thiscall" fn(*const IServerEntity) -> c_int = std::mem::transmute(vtable.add(6).read());
            get_idx(self)
        }
    }
}

/// The core server-side entity class in the Source Engine.
#[repr(C)] pub struct CBaseEntity { _private: [u8; 0] }

impl CBaseEntity {
    /// Casts this entity to its `IServerEntity` interface safely.
    pub fn as_server_entity(&self) -> &IServerEntity {
        unsafe { &*(self as *const _ as *const IServerEntity) }
    }

    pub fn get_networkable(&self) -> Option<&mut IServerNetworkable> {
        self.as_server_entity().get_networkable()
    }

    /// Casts this entity to its mutable `IServerEntity` interface.
    pub fn as_server_entity_mut(&mut self) -> &mut IServerEntity {
        unsafe { &mut *(self as *mut _ as *mut IServerEntity) }
    }

    /// Returns the entity index (extracted from the handle).
    pub fn get_index(&self) -> i32 {
        self.as_server_entity().get_handle().0 as i32 & 0xFFF
    }

    /// Wrapper for IServerTools::GetKeyValue.
    pub fn get_key_value(&self, key: &str) -> Option<String> {
        let tools = crate::get_engine().server_tools();
        tools.get_key_value(self, key)
    }

    /// Shortcut: Gets the network Edict directly from the entity.
    pub fn get_edict<'a>(&self) -> Option<&'a mut Edict> {
        self.as_server_entity().get_networkable()?.get_edict()
    }

    /// Shortcut: Gets the C++ Class Name directly from the entity.
    pub fn get_class_name(&self) -> String {
        if let Some(net) = self.as_server_entity().get_networkable() {
            net.get_class_name()
        } else {
            String::new()
        }
    }

    /// Shortcut: Gets the Network Server Class directly.
    pub fn get_server_class<'a>(&self) -> Option<&'a mut ServerClass> {
        self.as_server_entity().get_networkable()?.get_server_class()
    }

    //
    // High-level entity manipulation methods
    //

    /// Retrieves the networkable class name of the entity.
    pub fn get_classname(&self) -> String {
        self.get_class_name()
    }

    /// Reads the current health of the entity via the engine's DataMap.
    pub fn get_health(&self) -> i32 {
        let tools = crate::get_engine().server_tools();
        if let Some(val) = tools.get_key_value(self, "health") {
            val.parse().unwrap_or(0)
        } else {
            0
        }
    }

    /// Returns the current absolute world coordinates (origin) of the entity.
    pub fn get_origin(&self) -> Vector {
        let tools = crate::get_engine().server_tools();
        if let Some(val) = tools.get_key_value(self, "origin") {
            // The string format is typically: "X Y Z"
            let parts: Vec<&str> = val.split_whitespace().collect();
            if parts.len() >= 3 {
                return Vector {
                    x: parts[0].parse().unwrap_or(0.0),
                    y: parts[1].parse().unwrap_or(0.0),
                    z: parts[2].parse().unwrap_or(0.0),
                };
            }
        }
        Vector::default()
    }

    /// Returns the rotation angles of the entity (Pitch, Yaw, Roll).
    pub fn get_angles(&self) -> QAngle {
        let tools = crate::get_engine().server_tools();
        if let Some(val) = tools.get_key_value(self, "angles") {
            let parts: Vec<&str> = val.split_whitespace().collect();
            if parts.len() >= 3 {
                return QAngle {
                    x: parts[0].parse().unwrap_or(0.0),
                    y: parts[1].parse().unwrap_or(0.0),
                    z: parts[2].parse().unwrap_or(0.0),
                };
            }
        }
        QAngle::default()
    }

    /// Returns the target name ("targetname") of the entity.
    pub fn get_name(&self) -> String {
        let tools = crate::get_engine().server_tools();
        tools.get_key_value(self, "targetname").unwrap_or_default()
    }

    /// Removes the entity from the world using IServerTools.
    pub fn destroy(&self) {
        let tools = crate::get_engine().server_tools();
        if let Some(hammer_id_str) = tools.get_key_value(self, "hammerid") {
            if let Ok(hammer_id) = hammer_id_str.parse::<i32>() {
                tools.remove_entity(hammer_id);
            }
        }
    }

    /// Sets a string key-value field for the entity.
    pub fn set_key_value(&mut self, key: &str, value: &str) -> bool {
        let tools = crate::get_engine().server_tools();
        tools.set_key_value_str(self, key, value)
    }

    /// Sets an integer key-value field for the entity.
    pub fn set_key_value_int(&mut self, key: &str, value: i32) -> bool {
        let tools = crate::get_engine().server_tools();
        tools.set_key_value_flt(self, key, value as f32)
    }
}

/// Holds information about an entity being respawned with edits (used by IServerTools).
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CEntityRespawnInfo {
    pub hammer_id: c_int,
    pub ent_text: *const c_char,
}

impl CEntityRespawnInfo {
    /// Safely retrieves the entity text (KeyValues block) as a Rust String.
    pub fn entity_text(&self) -> String {
        if self.ent_text.is_null() {
            return String::new();
        }
        unsafe { CStr::from_ptr(self.ent_text).to_string_lossy().into_owned() }
    }
}

#[repr(C)] pub struct Edict { _private: [u8; 0] }
#[repr(C)] pub struct IClientEntity { _private: [u8; 0] }
#[repr(C)] pub struct ICollideable { _private: [u8; 0] }
#[repr(C)] pub struct IChangeInfoAccessor { _private: [u8; 0] }
#[repr(C)] pub struct ISpatialPartition { _private: [u8; 0] }
#[repr(C)] pub struct IScratchPad3D { _private: [u8; 0] }
#[repr(C)] pub struct CCheckTransmitInfo { _private: [u8; 0] }
#[repr(C)] pub struct CSharedEdictChangeInfo { _private: [u8; 0] }
