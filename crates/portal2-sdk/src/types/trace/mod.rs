pub mod masks;

pub use masks::*;

use std::ffi::{c_char, c_int, c_void};
use super::{Vector, QAngle, CBaseEntity};

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

#[repr(C)]
pub struct csurface_t {
    pub name: *const c_char,
    pub surface_props: i16,
    pub flags: u16,
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
pub enum TraceType_t {
    Everything = 0,
    WorldOnly,
    EntitiesOnly,
    EverythingFilterProps,
}

// ITraceFilter VTable bridge
#[repr(C)]
pub struct ITraceFilterVTable {
    pub should_hit_entity: unsafe extern "thiscall" fn(this: *mut c_void, entity: *mut c_void, mask: c_int) -> bool,
    pub get_trace_type: unsafe extern "thiscall" fn(this: *mut c_void) -> TraceType_t,
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
    let filter = &*(_this as *const TraceFilter);
    entity != filter.skip
}

unsafe extern "thiscall" fn trace_filter_get_trace_type(_this: *mut c_void) -> TraceType_t {
    TraceType_t::Everything
}

impl TraceFilter {
    pub fn new(skip_entity: Option<&CBaseEntity>) -> Self {
        Self {
            vtable: &TRACE_FILTER_VTABLE,
            skip: skip_entity.map_or(std::ptr::null_mut(), |e| e as *const _ as *mut c_void),
        }
    }
}
