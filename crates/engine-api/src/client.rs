use std::ffi::{c_char, c_int, CStr, CString};

use crate::types::ModelT;

use super::types::{PlayerInfo, QAngle};

// Opaque type for the `this` pointer.
#[repr(C)] pub(crate) struct RawIVEngineClient { _private: [u8; 0] }

type FnGetScreenSize = unsafe extern "thiscall" fn(this: *const IVEngineClient, w: *mut c_int, h: *mut c_int);
type FnServerCmd = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient, cmd: *const c_char, reliable: bool);
type FnClientCmd = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient, cmd: *const c_char);
type FnGetPlayerInfo = unsafe extern "thiscall" fn(this: *mut RawIVEngineClient, ent_num: c_int, p_info: *mut PlayerInfo) -> bool;
type FnGetPlayerForUserID = unsafe extern "thiscall" fn(this: *const RawIVEngineClient, user_id: c_int) -> c_int;
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
type FnConIsVisible = unsafe extern "thiscall" fn(this: *const RawIVEngineClient) -> bool;
type FnGetLocalPlayer = unsafe extern "thiscall" fn(this: *const RawIVEngineClient) -> c_int;
type FnLoadModel = unsafe extern "thiscall" fn(this: *const RawIVEngineClient, name: *const c_char, is_prop: bool) -> *const ModelT;
type FnKeyLookupBinding = unsafe extern "thiscall" fn(this: *const RawIVEngineClient, binding: *const c_char) -> *const c_char;
type FnIsPaused = unsafe extern "thiscall" fn(this: *const RawIVEngineClient) -> bool;

/// Represents an instance of the IVEngineClient interface.
/// Instead of a vtable, it holds a 'this' pointer to the C++ object
/// and direct pointers to the functions we need.
pub struct IVEngineClient {
    pub(crate) this: *mut RawIVEngineClient,

    pub(crate) get_screen_size: FnGetScreenSize,
    pub(crate) server_cmd: FnServerCmd,
    pub(crate) client_cmd: FnClientCmd,
    pub(crate) get_player_info: FnGetPlayerInfo,
    pub(crate) get_player_for_user_id: FnGetPlayerForUserID,
    pub(crate) get_last_time_stamp: FnGetLastTimeStamp,
    pub(crate) get_view_angles: FnGetViewAngles,
    pub(crate) set_view_angles: FnSetViewAngles,
    pub(crate) get_max_clients: FnGetMaxClients,
    pub(crate) is_in_game: FnIsInGame,
    pub(crate) is_connected: FnIsConnected,
    pub(crate) is_paused: FnIsPaused,
    pub(crate) is_drawing_loading_image: FnIsDrawingLoadingImage,
    pub(crate) get_level_name: FnGetLevelName,
    pub(crate) get_level_name_short: FnGetLevelName,
    pub(crate) execute_client_cmd_unrestricted: FnExecuteClientCmdUnrestricted,
    pub(crate) is_singlplayer: FnIsSinglplayer,
    pub(crate) con_is_visible: FnConIsVisible,
    pub(crate) get_local_player: FnGetLocalPlayer,
    pub(crate) load_model: FnLoadModel,
    pub(crate) key_lookup_binding: FnKeyLookupBinding,
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
        let c_str = CString::new(cmd).unwrap();
        unsafe { (self.server_cmd)(self.this, c_str.as_ptr(), reliable) };
    }

    /// Executes a command on the client side (with potential restrictions).
    pub fn client_cmd(&self, cmd: &str) {
        let c_str = CString::new(cmd).unwrap();
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

    /// Returns the player's current view angles.
    pub fn get_view_angles(&self) -> QAngle {
        let mut angles = QAngle::default();
        // SAFETY: `this` is a valid pointer. The engine function will write
        // the current view angles into the `angles` struct we provide.
        unsafe { (self.get_view_angles)(self.this, &mut angles) };
        angles
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

    /// Is the game paused?
    pub fn is_connected(&self) -> bool {
        unsafe { (self.is_connected)(self.this) }
    }

    /// Returns `true` if game in pause.
    pub fn is_paused(&self) -> bool {
        unsafe { (self.is_paused)(self.this) }
    }

    /// Returns the name of the current map (e.g., "maps/de_dust2.bsp").
    pub fn get_level_name(&self) -> String {
        unsafe {
            let c_str_ptr = (self.get_level_name)(self.this);
            if c_str_ptr.is_null() {
                return String::new();
            }
            CStr::from_ptr(c_str_ptr).to_string_lossy().into_owned()
        }
    }

    /// Returns the name of the current map (e.g., "de_dust2").
    pub fn get_level_name_short(&self) -> String {
        unsafe {
            let c_str_ptr = (self.get_level_name_short)(self.this);
            if c_str_ptr.is_null() {
                return String::new();
            }
            CStr::from_ptr(c_str_ptr).to_string_lossy().into_owned()
        }
    }

    /// Executes a client command without restrictions.
    pub fn execute_client_cmd_unrestricted(&self, cmd: &str) {
        if let Ok(c_str) = CString::new(cmd) {
            // SAFETY: `this` is valid, c_str is valid for this scope.
            unsafe { (self.execute_client_cmd_unrestricted)(self.this, c_str.as_ptr()) };
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

    /// Returns `true` if the console is visible.
    pub fn con_is_visible(&self) -> bool {
        // SAFETY: `this` is guaranteed to be a valid pointer.
        unsafe { (self.con_is_visible)(self.this as *const _) }
    }

    /// Returns the entity index of the local player.
    pub fn get_local_player(&self) -> i32 {
        // SAFETY: `this` is guaranteed to be a valid pointer.
        unsafe { (self.get_local_player)(self.this as *const _) as i32 }
    }

    /// Loads a model by its name.
    /// Returns a pointer to the model if successful, otherwise `None`.
    pub fn load_model(&self, name: &str, is_prop: bool) -> Option<*const ModelT> {
        let c_str = match CString::new(name) {
            Ok(s) => s,
            Err(_) => return None, // Return None if the string contains a null byte
        };

        // SAFETY: `this` is a valid pointer.
        // `c_str` is a valid, null-terminated C-style string.
        let model_ptr = unsafe { (self.load_model)(self.this as *const _, c_str.as_ptr(), is_prop) };

        if model_ptr.is_null() {
            None
        } else {
            Some(model_ptr)
        }
    }

    /// Looks up the key binding for a given command.
    /// Returns the key name if found, otherwise an empty string.
    pub fn key_lookup_binding(&self, binding: &str) -> String {
        let c_str = match CString::new(binding) {
            Ok(s) => s,
            Err(_) => return String::new(), // Return an empty string if there's a null byte
        };

        // SAFETY: `this` is a valid pointer.
        // `c_str` is a valid, null-terminated C-style string.
        let result_ptr = unsafe { (self.key_lookup_binding)(self.this as *const _, c_str.as_ptr()) };

        if result_ptr.is_null() {
            return String::new();
        }

        unsafe {
            CStr::from_ptr(result_ptr).to_string_lossy().into_owned()
        }
    }

    /// Gets the screen dimensions.
    /// Returns a tuple of (width, height).
    pub fn get_screen_size(&self) -> (i32, i32) {
        let mut width: c_int = 0;
        let mut height: c_int = 0;

        // SAFETY: `this` is a valid pointer.
        // `&mut width` and `&mut height` are valid pointers to `c_int`.
        unsafe { (self.get_screen_size)(self.this as *const _, &mut width, &mut height) };

        (width as i32, height as i32)
    }

    /// Gets the player's entity index for a given user ID.
    pub fn get_player_for_user_id(&self, user_id: i32) -> i32 {
        // SAFETY: `this` is guaranteed to be a valid pointer.
        unsafe { (self.get_player_for_user_id)(self.this as *const _, user_id as c_int) as i32 }
    }
}
