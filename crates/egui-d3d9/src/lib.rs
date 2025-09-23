macro_rules! expect {
    ($val:expr, $msg:expr) => {
        if cfg!(feature = "silent") {
            $val.unwrap()
        } else {
            $val.expect($msg)
        }
    };
}

pub mod app;
pub mod inputman;
pub mod mesh;
pub mod state;
pub mod texman;

pub use app::*;
