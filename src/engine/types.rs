use std::ffi::{c_char, c_int, CStr};

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
    pub version: u64,
    pub xuid: u64,
    pub name: [c_char; 128], // MAX_PLAYER_NAME_LENGTH
    pub user_id: c_int,
    pub guid: [c_char; 33],
    pub friends_id: u32,
    pub friends_name: [c_char; 128],
    pub fake_player: bool,
    pub is_hltv: bool,
    pub custom_files: [u32; 4],
    pub files_downloaded: u8,
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            version: 0,
            xuid: 0,
            name: [0; 128],
            user_id: 0,
            guid: [0; 33],
            friends_id: 0,
            friends_name: [0; 128],
            fake_player: false,
            is_hltv: false,
            custom_files: [0; 4],
            files_downloaded: 0,
        }
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

    /// Returns the player's GUID as a Rust String.
    pub fn guid(&self) -> String {
        unsafe {
            CStr::from_ptr(self.guid.as_ptr())
                .to_string_lossy()
                .into_owned()
        }
    }

    /// Returns the player's friend's name as a Rust String.
    pub fn friends_name(&self) -> String {
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
