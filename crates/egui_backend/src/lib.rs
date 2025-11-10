//! egui_backend: A D3D9 backend for egui.
//!
//! This crate provides a D3D9 backend for the egui library.
//! It is used to render egui UIs in a D3D9 application.
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

// Lightweight backend (no generic user state)
pub mod lite;

pub use lite::EguiDx9Lite;
