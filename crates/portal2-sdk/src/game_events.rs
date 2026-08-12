use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{OnceLock, RwLock};
use crate::platform::abi::{vfn, vfn_impl};

/// A unique ID for a registered listener. Use this to unregister later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListenerId(usize);

/// A safe wrapper around a Source Engine Game Event.
pub struct GameEvent<'a> {
    raw: &'a IGameEvent,
}

impl<'a> GameEvent<'a> {
    pub fn name(&self) -> &str {
        unsafe {
            let name_ptr = self.raw.get_name();
            if name_ptr.is_null() {
                return "";
            }
            CStr::from_ptr(name_ptr).to_str().unwrap_or("")
        }
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        let key_c = std::ffi::CString::new(key).unwrap_or_default();
        unsafe { self.raw.get_bool(key_c.as_ptr(), default) }
    }

    pub fn get_int(&self, key: &str, default: i32) -> i32 {
        let key_c = std::ffi::CString::new(key).unwrap_or_default();
        unsafe { self.raw.get_int(key_c.as_ptr(), default) }
    }

    pub fn get_uint64(&self, key: &str, default: u64) -> u64 {
        let key_c = std::ffi::CString::new(key).unwrap_or_default();
        unsafe { self.raw.get_uint64(key_c.as_ptr(), default) }
    }

    pub fn get_float(&self, key: &str, default: f32) -> f32 {
        let key_c = std::ffi::CString::new(key).unwrap_or_default();
        unsafe { self.raw.get_float(key_c.as_ptr(), default) }
    }

    pub fn get_string(&self, key: &str, default: &str) -> String {
        let key_c = std::ffi::CString::new(key).unwrap_or_default();
        let def_c = std::ffi::CString::new(default).unwrap_or_default();
        unsafe {
            let ptr = self.raw.get_string(key_c.as_ptr(), def_c.as_ptr());
            if ptr.is_null() {
                return default.to_string();
            }
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }
}

// --- INTERNAL BRIDGE ---

/// A closure type for game event callbacks.
pub type EventCallback = Box<dyn Fn(&GameEvent) + Send + Sync + 'static>;

/// Registry stores callbacks with their IDs indexed by event name.
static EVENT_REGISTRY: OnceLock<RwLock<HashMap<String, Vec<(ListenerId, EventCallback)>>>> = OnceLock::new();
static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

fn get_registry() -> &'static RwLock<HashMap<String, Vec<(ListenerId, EventCallback)>>> {
    EVENT_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

#[repr(C)]
pub struct MasterListener {
    vtable: *const IGameEventListener2Vtable,
}

static MASTER_LISTENER_VTABLE: IGameEventListener2Vtable = IGameEventListener2Vtable {
    destructor: master_listener_destructor,
    fire_game_event: master_listener_fire_game_event,
    get_event_debug_id: master_listener_get_debug_id,
};

vfn_impl! {
    fn master_listener_fire_game_event(_this: *mut c_void, event: *mut IGameEvent) {
        if event.is_null() { return; }
        let safe_event = GameEvent { raw: unsafe { &*event } };
        let event_name = safe_event.name();

        // Dispatch ONLY to listeners interested in THIS specific event
        if let Ok(registry) = get_registry().read() {
            if let Some(callbacks) = registry.get(event_name) {
                for (_, callback) in callbacks {
                    (callback)(&safe_event);
                }
            }
        }
    }

    fn master_listener_destructor(_this: *mut c_void, _flags: i32) {}

    fn master_listener_get_debug_id(_this: *mut c_void) -> i32 { 42 }
}

// FFI STRUCTURES

#[repr(C)]
pub(crate) struct IGameEvent {
    vtable: *const IGameEventVtable,
}

#[allow(unsafe_op_in_unsafe_fn)]
impl IGameEvent {
    unsafe fn get_name(&self) -> *const c_char { ((*self.vtable).get_name)(self as *const _ as *mut _) }
    unsafe fn get_bool(&self, key: *const c_char, def: bool) -> bool { ((*self.vtable).get_bool)(self as *const _ as *mut _, key, def) }
    unsafe fn get_int(&self, key: *const c_char, def: i32) -> i32 { ((*self.vtable).get_int)(self as *const _ as *mut _, key, def) }
    unsafe fn get_uint64(&self, key: *const c_char, def: u64) -> u64 { ((*self.vtable).get_uint64)(self as *const _ as *mut _, key, def) }
    unsafe fn get_float(&self, key: *const c_char, def: f32) -> f32 { ((*self.vtable).get_float)(self as *const _ as *mut _, key, def) }
    unsafe fn get_string(&self, key: *const c_char, def: *const c_char) -> *const c_char { ((*self.vtable).get_string)(self as *const _ as *mut _, key, def) }
}

#[repr(C)]
struct IGameEventVtable {
    destructor: vfn!((this: *mut c_void, flags: i32)),
    get_name: vfn!((this: *mut c_void) -> *const c_char),
    is_reliable: vfn!((this: *mut c_void) -> bool),
    is_local: vfn!((this: *mut c_void) -> bool),
    is_empty: vfn!((this: *mut c_void, key_name: *const c_char) -> bool),
    get_bool: vfn!((this: *mut c_void, key_name: *const c_char, default_val: bool) -> bool),
    get_int: vfn!((this: *mut c_void, key_name: *const c_char, default_val: i32) -> i32),
    get_uint64: vfn!((this: *mut c_void, key_name: *const c_char, default_val: u64) -> u64),
    get_float: vfn!((this: *mut c_void, key_name: *const c_char, default_val: f32) -> f32),
    get_string: vfn!((this: *mut c_void, key_name: *const c_char, default_val: *const c_char) -> *const c_char),
    set_bool: vfn!((this: *mut c_void, key_name: *const c_char, val: bool)),
    set_int: vfn!((this: *mut c_void, key_name: *const c_char, val: i32)),
    set_uint64: vfn!((this: *mut c_void, key_name: *const c_char, val: u64)),
    set_float: vfn!((this: *mut c_void, key_name: *const c_char, val: f32)),
    set_string: vfn!((this: *mut c_void, key_name: *const c_char, val: *const c_char)),
}

#[repr(C)]
pub struct IGameEventManager2 {
    pub(crate) this: *mut c_void,
    pub(crate) add_listener: vfn!((this: *mut c_void, listener: *mut c_void, name: *const c_char, server_side: bool) -> bool),
    pub(crate) remove_listener: vfn!((this: *mut c_void, listener: *mut c_void)),
    /// The listener instance. Injected during initialization.
    pub(crate) listener: *mut MasterListener,
}

impl IGameEventManager2 {
    /// Registers a closure and returns a ListenerId.
    pub fn listen<F>(&self, event_name: &str, callback: F) -> ListenerId
    where
        F: Fn(&GameEvent) + Send + Sync + 'static
    {
        let mut registry = get_registry().write().unwrap();

        // If this is the first time we listen to this event, tell the engine
        if !registry.contains_key(event_name) {
            let name_c = std::ffi::CString::new(event_name).unwrap();
            unsafe {
                (self.add_listener)(self.this, self.listener as *mut c_void, name_c.as_ptr(), true);
            }
        }

        let id = ListenerId(NEXT_ID.fetch_add(1, Ordering::SeqCst));
        registry.entry(event_name.to_string()).or_default().push((id, Box::new(callback)));
        id
    }

    /// Removes a specific listener by its ID.
    pub fn unlisten(&self, id: ListenerId) {
        let mut registry = get_registry().write().unwrap();
        for listeners in registry.values_mut() {
            listeners.retain(|(lid, _)| *lid != id);
        }
    }

    /// Removes all Rust listeners for a specific event name.
    pub fn unlisten_all(&self, event_name: &str) {
        if let Ok(mut registry) = get_registry().write() {
            registry.remove(event_name);
        }
    }

    /// Completely unregisters the listener from the engine.
    pub fn shutdown_all_listeners(&self) { // TODO: Use this only when unloading the entire overlay system
        unsafe {
            (self.remove_listener)(self.this, self.listener as *mut c_void);
        }
        if let Ok(mut registry) = get_registry().write() {
            registry.clear();
        }
    }
}

#[repr(C)]
struct IGameEventListener2Vtable {
    pub destructor: vfn!((this: *mut c_void, flags: i32)),
    pub fire_game_event: vfn!((this: *mut c_void, event: *mut IGameEvent)),
    pub get_event_debug_id: vfn!((this: *mut c_void) -> i32),
}

pub fn create_master_listener() -> *mut MasterListener {
    Box::into_raw(Box::new(MasterListener {
        vtable: &MASTER_LISTENER_VTABLE,
    }))
}
