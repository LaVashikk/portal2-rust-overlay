use std::ffi::{c_char, c_int, CStr};

const MAX_PLAYER_NAME_LENGTH: usize = 128;
const SIGNED_GUID_LEN: usize = 32;

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct QAngle {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct VMatrix {
    pub m: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct PlayerInfo {
    // SteamID64
    pub xuid: u64,

    // Player name
    pub name: [c_char; MAX_PLAYER_NAME_LENGTH],

    // Unique ID on the server (1, 2, 3, etc.)
    pub user_id: c_int,

    // SteamID2 as a string ("STEAM_X:Y:Z")
    pub guid: [c_char; SIGNED_GUID_LEN + 1], // +1 for null terminator

    // Other fields, order may be important
    pub friends_id: u32,
    pub friends_name: [c_char; MAX_PLAYER_NAME_LENGTH],
    pub fake_player: bool,   // Is this a bot?
    pub is_hltv: bool,       // Is this an HLTV bot/proxy?
    pub custom_files: [u32; 4],
    pub files_downloaded: u8,

    _padding: [u8; 2],
}

impl Default for PlayerInfo {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

impl PlayerInfo {
    /// Returns the player's name as a Rust String.
    pub fn name(&self) -> String {
        unsafe {
            CStr::from_ptr(self.name.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Returns the GUID (SteamID2) as a Rust String.
    pub fn guid(&self) -> String {
        unsafe {
            CStr::from_ptr(self.guid.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Returns the player's friend's name as a Rust String.
    pub fn friends_name(&self) -> String { // todo? what is this?
        unsafe {
            CStr::from_ptr(self.friends_name.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }
}

// TODO: Add comments explaining these opaque types.
#[repr(C)] pub struct ModelT { _private: [u8; 0] }
#[repr(C)] pub struct ClientTextMessage { _private: [u8; 0] }
#[repr(C)] pub struct CSentence { _private: [u8; 0] }
#[repr(C)] pub struct CAudioSource { _private: [u8; 0] }
#[repr(C)] pub struct ISpatialQuery { _private: [u8; 0] }
#[repr(C)] pub struct IMaterialSystem { _private: [u8; 0] }
#[repr(C)] pub struct INetChannelInfo { _private: [u8; 0] }
#[repr(C)] pub struct KeyValues { _private: [u8; 0] }
#[repr(C)] pub struct IAchievementMgr { _private: [u8; 0] }
#[repr(C)] pub struct CGamestatsData { _private: [u8; 0] }
