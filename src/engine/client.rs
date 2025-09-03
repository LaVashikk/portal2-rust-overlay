use std::ffi::{c_char, c_int};

use crate::engine::types::{PlayerInfo, QAngle};

// Opaque type for the `this` pointer.
#[repr(C)] pub(crate) struct RawIVEngineClient { _private: [u8; 0] }

type FnServerCmd = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient, cmd: *const c_char, reliable: bool);
type FnClientCmd = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient, cmd: *const c_char);
type FnGetPlayerInfo = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient, ent_num: c_int, p_info: *mut PlayerInfo) -> bool;
type FnGetLastTimeStamp = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient) -> f32;
type FnGetViewAngles = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient, va: *mut QAngle);
type FnSetViewAngles = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient, va: *const QAngle);
type FnGetMaxClients = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient) -> c_int;
type FnIsInGame = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient) -> bool;
type FnIsConnected = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient) -> bool;
type FnIsDrawingLoadingImage = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient) -> bool;
type FnGetLevelName = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient) -> *const c_char;
type FnExecuteClientCmdUnrestricted = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient, cmd: *const c_char);
type FnIsSinglplayer = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient) -> bool;

/// Represents an instance of the IVEngineClient interface.
/// Instead of a vtable, it holds a 'this' pointer to the C++ object
/// and direct pointers to the functions we need.
pub struct IVEngineClient {
    pub(crate) this: *mut RawIVEngineClient,

    pub(crate) server_cmd: FnServerCmd,
    pub(crate) client_cmd: FnClientCmd,
    pub(crate) get_player_info: FnGetPlayerInfo,
    pub(crate) get_last_time_stamp: FnGetLastTimeStamp,
    pub(crate) get_view_angles: FnGetViewAngles,
    pub(crate) set_view_angles: FnSetViewAngles,
    pub(crate) get_max_clients: FnGetMaxClients,
    pub(crate) is_in_game: FnIsInGame,
    pub(crate) is_connected: FnIsConnected,
    pub(crate) is_drawing_loading_image: FnIsDrawingLoadingImage,
    pub(crate) get_level_name: FnGetLevelName,
    pub(crate) execute_client_cmd_unrestricted: FnExecuteClientCmdUnrestricted,
    pub(crate) is_singlplayer: FnIsSinglplayer,
}

/// This implementation provides safe, idiomatic Rust methods to interact with the game's engine client interface.
/// All unsafe Foreign Function Interface (FFI) calls are wrapped within these methods.
///
/// SAFETY: The `this` pointer is guaranteed to be valid for the lifetime of `IVEngineClient`.
/// The `c_str` pointer is valid as it's derived from a CString that lives until the end of this function.
/// The external function is a valid game function found via signature scanning.
impl IVEngineClient {
    /// Sends a command to the server.
    pub fn server_cmd(&self, cmd: &str, reliable: bool) {
        let c_str = std::ffi::CString::new(cmd).unwrap();
        unsafe { (self.server_cmd)(self.this, c_str.as_ptr(), reliable) };
    }

    /// Executes a command on the client side (with potential restrictions).
    pub fn client_cmd(&self, cmd: &str) {
        let c_str = std::ffi::CString::new(cmd).unwrap();
        unsafe { (self.client_cmd)(self.this, c_str.as_ptr()) };
    }

    /// Retrieves information about a player by their entity index.
    /// Indices start at 1.
    pub fn get_player_info(&self, ent_num: i32) -> Option<PlayerInfo> {
        let mut info = PlayerInfo::default();
        let success = unsafe {
            (self.get_player_info)(self.this, ent_num as c_int, &mut info)
        };
        if success { Some(info) } else { None }
    }

    /// Returns the timestamp of the last packet received from the server.
    pub fn get_last_time_stamp(&self) -> f32 {
        unsafe { (self.get_last_time_stamp)(self.this) }
    }

    /// Gets the player's current view angles.
    pub fn get_view_angles(&self, angles: &mut QAngle) {
        unsafe { (self.get_view_angles)(self.this, angles) };
    }

    /// Sets the player's view angles.
    pub fn set_view_angles(&self, angles: &QAngle) {
        unsafe { (self.set_view_angles)(self.this, angles) };
    }

    /// Returns the maximum number of clients on the server.
    pub fn get_max_clients(&self) -> i32 {
        unsafe { (self.get_max_clients)(self.this) as i32 }
    }

    /// Returns `true` if the player is fully connected and in the game.
    pub fn is_in_game(&self) -> bool {
        unsafe { (self.is_in_game)(self.this) }
    }

    /// Returns `true` if the player is connected to a server (may still be loading).
    pub fn is_connected(&self) -> bool {
        unsafe { (self.is_connected)(self.this) }
    }

    /// Returns the name of the current map (e.g., "de_dust2").
    pub fn get_level_name(&self) -> String {
        unsafe {
            let c_str_ptr = (self.get_level_name)(self.this);
            std::ffi::CStr::from_ptr(c_str_ptr).to_string_lossy().into_owned()
        }
    }

    /// Executes a client command without restrictions.
    pub fn execute_client_cmd_unrestricted(&self, cmd: &str) {
        if let Ok(c_str) = std::ffi::CString::new(cmd) {
            // SAFETY: `this` is valid, c_str is valid for this scope.
            unsafe { (self.execute_client_cmd_unrestricted)(self.this, c_str.as_ptr()) };
        } else {
            log::error!("Attempted to execute a command with a null byte: {}", cmd);
        }
    }

    /// Returns `true` if the game is in singleplayer mode.
    pub fn is_singlplayer(&self) -> bool {
        unsafe { (self.is_singlplayer)(self.this) }
    }

    /// Returns `true` if the game is currently showing a loading screen.
    pub fn is_loading_map(&self) -> bool {
        unsafe { (self.is_drawing_loading_image)(self.this) }
    }
}
