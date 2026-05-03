use crate::{server_tools::IServerTools, types::{CBaseEntity, Vector}};
use std::marker::PhantomData;

/// A convenient wrapper for entity searching and iteration.
pub struct Entities<'a> {
    tools: &'a IServerTools,
}

impl<'a> Entities<'a> {
    pub fn new(tools: &'a IServerTools) -> Self {
        Self { tools }
    }

    // --------------------------------------------------------------------
    // VScript-style API
    // --------------------------------------------------------------------

    pub fn first(&self) -> Option<&mut CBaseEntity> {
        self.tools.first_entity()
    }

    pub fn next(&self, prev: &CBaseEntity) -> Option<&mut CBaseEntity> {
        self.tools.next_entity(prev)
    }

    /// Finds the next entity matching the specified classname.
    pub fn find_by_classname(&self, prev: Option<&CBaseEntity>, classname: &str) -> Option<&mut CBaseEntity> {
        let mut current = match prev {
            Some(ent) => self.tools.next_entity(ent),
            None => self.tools.first_entity(),
        };

        while let Some(ent) = current {
            if ent.get_classname() == classname {
                // Unsafe is required here to bypass the borrow checker's limitation
                // when returning a mutable reference directly from a loop.
                return Some(unsafe { &mut *(ent as *mut CBaseEntity) });
            }
            current = self.tools.next_entity(ent);
        }
        None
    }

    /// Finds the next entity matching the specified targetname.
    pub fn find_by_name(&self, prev: Option<&CBaseEntity>, targetname: &str) -> Option<&mut CBaseEntity> {
        let mut current = match prev {
            Some(ent) => self.tools.next_entity(ent),
            None => self.tools.first_entity(),
        };

        while let Some(ent) = current {
            if ent.get_name() == targetname {
                return Some(unsafe { &mut *(ent as *mut CBaseEntity) });
            }
            current = self.tools.next_entity(ent);
        }
        None
    }

    /// Finds the next entity within the specified radius around a center point.
    pub fn find_in_sphere(&self, prev: Option<&CBaseEntity>, center: &Vector, radius: f32) -> Option<&mut CBaseEntity> {
        let mut current = match prev {
            Some(ent) => self.tools.next_entity(ent),
            None => self.tools.first_entity(),
        };

        while let Some(ent) = current {
            let origin = ent.get_origin();
            if origin.distance(center) <= radius {
                return Some(unsafe { &mut *(ent as *mut CBaseEntity) });
            }
            current = self.tools.next_entity(ent);
        }
        None
    }

    // --------------------------------------------------------------------
    // Standard Rust iterators
    // --------------------------------------------------------------------

    /// Returns an iterator over all entities on the server.
    /// Enables the use of standard Rust iterator adapters (e.g., `.filter()`, `.map()`, `for` loops).
    pub fn iter(&self) -> EntityIter<'a> {
        EntityIter {
            tools: self.tools,
            current_ptr: std::ptr::null_mut(),
            started: false,
            _marker: PhantomData,
        }
    }
}

/// Safe iterator over all server entities.
pub struct EntityIter<'a> {
    tools: &'a IServerTools,
    current_ptr: *mut std::ffi::c_void,
    started: bool,
    _marker: PhantomData<&'a mut CBaseEntity>,
}

impl<'a> Iterator for EntityIter<'a> {
    type Item = &'a mut CBaseEntity;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if !self.started {
                self.started = true;

                if let Some(ent) = self.tools.first_entity() {
                    self.current_ptr = ent as *mut _ as *mut std::ffi::c_void;
                    return Some(&mut *(self.current_ptr as *mut CBaseEntity));
                }
                None
            } else {
                if self.current_ptr.is_null() {
                    return None;
                }

                // Cast the raw pointer back to a reference to fetch the next entity
                let prev_ref = &*(self.current_ptr as *const CBaseEntity);
                let next_ent = self.tools.next_entity(prev_ref);

                if let Some(ent) = next_ent {
                    self.current_ptr = ent as *mut _ as *mut std::ffi::c_void;
                    return Some(&mut *(self.current_ptr as *mut CBaseEntity));
                } else {
                    self.current_ptr = std::ptr::null_mut();
                    None
                }
            }
        }
    }
}
