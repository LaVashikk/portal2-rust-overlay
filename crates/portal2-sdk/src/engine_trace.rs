use std::ffi::{c_int, c_void};
use crate::types::{Ray_t, Trace_t, TraceFilter, Vector, ICollideable, MaskFlags, CBaseEntity};

// Opaque type for the `this` pointer.
#[repr(C)] pub(crate) struct RawIEngineTrace { _private: [u8; 0] }

type FnGetPointContents = unsafe extern "thiscall" fn(this: *mut RawIEngineTrace, abs_pos: *const Vector, mask: c_int, entity: *mut *mut c_void) -> c_int;
type FnClipRayToEntity = unsafe extern "thiscall" fn(this: *mut RawIEngineTrace, ray: *const Ray_t, mask: u32, entity: *mut c_void, trace: *mut Trace_t);
type FnTraceRay = unsafe extern "thiscall" fn(this: *mut RawIEngineTrace, ray: *const Ray_t, mask: u32, filter: *mut c_void, trace: *mut Trace_t);
type FnGetCollideable = unsafe extern "thiscall" fn(this: *mut RawIEngineTrace, entity: *mut c_void) -> *mut ICollideable;

/// Interface for ray tracing and collision testing.
pub struct IEngineTrace {
    pub(crate) this: *mut RawIEngineTrace,

    pub(crate) get_point_contents: FnGetPointContents,
    pub(crate) clip_ray_to_entity: FnClipRayToEntity,
    pub(crate) trace_ray: FnTraceRay,
    pub(crate) get_collideable: FnGetCollideable,
}

impl IEngineTrace {
    /// Returns the contents mask at a particular world-space position.
    pub fn get_point_contents(&self, pos: &Vector, mask: MaskFlags) -> i32 {
        unsafe { (self.get_point_contents)(self.this, pos, mask.bits() as c_int, std::ptr::null_mut()) as i32 }
    }

    /// Traces a ray against the world and entities using a filter.
    pub fn trace_ray(&self, ray: &Ray_t, mask: MaskFlags, filter: &mut TraceFilter) -> Trace_t {
        let mut trace = Trace_t::default();
        unsafe { (self.trace_ray)(self.this, ray, mask.bits() as u32, filter as *mut _ as *mut c_void, &mut trace) };
        trace
    }

    /// A convenience method for a simple line trace between two points.
    pub fn line_trace(&self, start: Vector, end: Vector, mask: MaskFlags, skip_entity: Option<&CBaseEntity>) -> Trace_t {
        let ray = Ray_t::new(start, end);
        let mut filter = TraceFilter::new(skip_entity);
        self.trace_ray(&ray, mask, &mut filter)
    }

    /// Traces a ray against a specific entity.
    pub fn clip_ray_to_entity(&self, ray: &Ray_t, mask: MaskFlags, entity: &CBaseEntity) -> Trace_t {
        let mut trace = Trace_t::default();
        unsafe { (self.clip_ray_to_entity)(self.this, ray, mask.bits() as u32, entity as *const _ as *mut _, &mut trace) };
        trace
    }

    /// Converts a handle entity to a collideable interface.
    pub fn get_collideable(&self, entity: &CBaseEntity) -> Option<&mut ICollideable> {
        unsafe {
            let ptr = (self.get_collideable)(self.this, entity as *const _ as *mut _);
            if ptr.is_null() { None } else { Some(&mut *ptr) }
        }
    }
}
