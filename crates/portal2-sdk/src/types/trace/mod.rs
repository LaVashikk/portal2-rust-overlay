/// Usage example:
/// ```
/// let e = engine.entities();
/// let local_player = e.find_by_classname(None, "player");
///
/// let server_tools = engine.server_tools();
/// if let Some((pos, angles)) = server_tools.get_player_position(None) {
///     let eye_pos = pos;
///     let forward = angles.to_forward_vector();
///     let end_pos = eye_pos + (forward * 8192.0);
///
///     let trace = engine.engine_trace().line_trace(eye_pos, end_pos, MaskFlags::SOLID, local_player.as_deref());
///
///     if trace.did_hit() {
///         if trace.did_hit_entity() {
///             log::info!("hit into: {}", trace.hit_entity().unwrap().get_class_name())
///         } else {
///             let surf = &trace.surface;
///             if surf.is_sky() {
///                 log::info!("Looking at sky: {}", surf.get_name());
///             } else if surf.is_no_portal() {
///                 log::info!("Can't place portal here (texture: {})", surf.get_name());
///             } else {
///                 log::info!("Looking at {}", surf.get_name());
///             }
///         }
///     }
/// ```
pub mod masks;

pub use masks::*;

use std::ffi::{c_char, c_int, c_void};
use super::{Vector, CBaseEntity};

#[repr(C, align(16))]
#[derive(Debug, Default, Clone, Copy)]
pub struct VectorAligned {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl From<Vector> for VectorAligned {
    fn from(v: Vector) -> Self {
        Self { x: v.x, y: v.y, z: v.z, w: 0.0 }
    }
}

#[repr(C)]
pub struct Ray_t {
    pub start: VectorAligned,
    pub delta: VectorAligned,
    pub start_offset: VectorAligned,
    pub extents: VectorAligned,
    pub world_axis_transform: *const c_void, // matrix3x4_t
    pub is_ray: bool,
    pub is_swept: bool,
}

impl Ray_t {
    pub fn new(start: Vector, end: Vector) -> Self {
        let delta = end - start;
        Self {
            start: start.into(),
            delta: delta.into(),
            start_offset: VectorAligned::default(),
            extents: VectorAligned::default(),
            world_axis_transform: std::ptr::null(),
            is_ray: true,
            is_swept: delta.length_sqr() != 0.0,
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitGroup {
    Generic = 0,
    Head = 1,
    Chest = 2,
    Stomach = 3,
    LeftArm = 4,
    RightArm = 5,
    LeftLeg = 6,
    RightLeg = 7,
    Gear = 10,
    Unknown = -1,
}

impl From<i32> for HitGroup {
    fn from(val: i32) -> Self {
        match val {
            0 => HitGroup::Generic,
            1 => HitGroup::Head,
            2 => HitGroup::Chest,
            3 => HitGroup::Stomach,
            4 => HitGroup::LeftArm,
            5 => HitGroup::RightArm,
            6 => HitGroup::LeftLeg,
            7 => HitGroup::RightLeg,
            10 => HitGroup::Gear,
            _ => HitGroup::Unknown,
        }
    }
}

#[repr(C)]
pub struct csurface_t {
    pub name: *const c_char,
    pub surface_props: i16,
    pub flags: u16,
}

impl csurface_t {
    /// Returns the name of the surface (e.g., texture/material name).
    pub fn get_name(&self) -> &str {
        if self.name.is_null() {
            return "";
        }
        unsafe {
            std::ffi::CStr::from_ptr(self.name)
                .to_str()
                .unwrap_or("")
        }
    }

    /// Returns the raw surface flags.
    pub fn get_flags(&self) -> SurfaceFlags {
        SurfaceFlags::from_bits_truncate(self.flags)
    }

    /// Returns true if this is a sky surface.
    pub fn is_sky(&self) -> bool {
        let flags = self.get_flags();
        flags.contains(SurfaceFlags::SKY) || flags.contains(SurfaceFlags::SKY2D)
    }

    /// Returns true if portals cannot be placed on this surface.
    pub fn is_no_portal(&self) -> bool {
        self.get_flags().contains(SurfaceFlags::NOPORTAL)
    }

    /// Returns true if paint/decals cannot be placed on this surface.
    pub fn is_no_paint(&self) -> bool {
        self.get_flags().contains(SurfaceFlags::NOPAINT)
    }

    /// Returns true if this surface is not drawn (nodraw).
    pub fn is_nodraw(&self) -> bool {
        self.get_flags().contains(SurfaceFlags::NODRAW)
    }
}

#[repr(C)]
pub struct cplane_t {
    pub normal: Vector,
    pub dist: f32,
    pub type_: u8,
    pub signbits: u8,
    pub pad: [u8; 2],
}

#[repr(C)]
pub struct Trace_t {
    pub startpos: Vector,
    pub endpos: Vector,
    pub plane: cplane_t,
    pub fraction: f32,
    pub contents: c_int,
    pub disp_flags: u16,
    pub allsolid: bool,
    pub startsolid: bool,
    pub fractionleftsolid: f32,
    pub surface: csurface_t,
    pub hitgroup: c_int,
    pub physicsbone: i16,
    pub world_surface_index: u16,
    pub entity: *mut CBaseEntity,
    pub hitbox: c_int,
}

impl Trace_t {
    /// Returns true if the trace hit something.
    pub fn did_hit(&self) -> bool {
        self.fraction < 1.0 || self.allsolid || self.startsolid
    }

    /// Returns true if the trace hit the world (no entity).
    pub fn did_hit_world(&self) -> bool {
        self.did_hit() && !self.did_hit_entity()
    }

    /// Returns true if the trace hit a specific entity.
    pub fn did_hit_entity(&self) -> bool {
        self.hit_entity().is_some_and(|ent| ent.get_class_name() != "worldspawn")
    }

    /// Returns the name of the material/texture hit by the trace.
    pub fn get_surface_name(&self) -> &str {
        self.surface.get_name()
    }

    /// Returns the hitgroup (e.g., Head, Chest) if an entity was hit.
    pub fn get_hitgroup(&self) -> HitGroup {
        HitGroup::from(self.hitgroup)
    }

    /// Safely returns a reference to the hit entity, if any.
    /// Returns `None` if the trace hit the world geometry or nothing.
    pub fn hit_entity(&self) -> Option<&CBaseEntity> {
        unsafe { self.entity.as_ref() }
    }

    /// Safely returns a mutable reference to the hit entity, if any.
    pub fn hit_entity_mut(&mut self) -> Option<&mut CBaseEntity> {
        unsafe { self.entity.as_mut() }
    }
}

impl Default for Trace_t {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(i32)]
pub enum TraceTypeT {
    Everything = 0,
    WorldOnly,
    EntitiesOnly,
    EverythingFilterProps,
}

// ITraceFilter VTable bridge
#[repr(C)]
pub struct ITraceFilterVTable {
    pub should_hit_entity: unsafe extern "thiscall" fn(this: *mut c_void, entity: *mut c_void, mask: c_int) -> bool,
    pub get_trace_type: unsafe extern "thiscall" fn(this: *mut c_void) -> TraceTypeT,
}

pub struct TraceFilter {
    pub vtable: *const ITraceFilterVTable,
    pub skip: *mut c_void,
}

static TRACE_FILTER_VTABLE: ITraceFilterVTable = ITraceFilterVTable {
    should_hit_entity: trace_filter_should_hit_entity,
    get_trace_type: trace_filter_get_trace_type,
};

unsafe extern "thiscall" fn trace_filter_should_hit_entity(_this: *mut c_void, entity: *mut c_void, _mask: c_int) -> bool {
    let filter = unsafe { &*(_this as *const TraceFilter) };
    entity != filter.skip
}

unsafe extern "thiscall" fn trace_filter_get_trace_type(_this: *mut c_void) -> TraceTypeT {
    TraceTypeT::Everything
}

impl TraceFilter {
    pub fn new(skip_entity: Option<&CBaseEntity>) -> Self {
        Self {
            vtable: &TRACE_FILTER_VTABLE,
            skip: skip_entity.map_or(std::ptr::null_mut(), |e| e as *const _ as *mut c_void),
        }
    }
}
