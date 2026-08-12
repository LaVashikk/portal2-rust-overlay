//! Calling-convention glue for the engine's C++ member functions.
//!
//! Every interface in this crate is a hand-written vtable, and the functions in it
//! are C++ non-static members: the `this` pointer is passed implicitly, and *how*
//! depends on the toolchain the game was built with.
//!
//! - **MSVC x86** (Windows build): `thiscall` - `this` is passed in `ECX`.
//! - **GCC x86** (native Linux build): plain `cdecl` - `this` is an ordinary
//!   leading stack argument.
//!
//! Either way the Rust-side signature is identical (`this` spelled out as the
//! first parameter), only the ABI string differs. Hence these macros, instead of
//! `#[cfg]`-duplicating ~240 declarations.
//!
//! # Caveat
//!
//! This equivalence breaks for functions returning a struct **by value**: MSVC
//! returns small PODs in `EAX`/`EDX`, while the i386 System V ABI always goes
//! through a hidden return-slot pointer. Such functions need per-platform
//! handling and cannot just be wrapped in `vfn!`.

/// Declares a vtable function-pointer *type* using the platform's member-function ABI.
///
/// ```ignore
/// type FnServerCmd = vfn!((this: *mut RawIVEngineClient, cmd: *const c_char));
/// type FnIsInGame  = vfn!((this: *mut RawIVEngineClient) -> bool);
/// ```
#[cfg(target_os = "windows")]
macro_rules! vfn {
    ($($signature:tt)*) => { unsafe extern "thiscall" fn $($signature)* };
}

#[cfg(not(target_os = "windows"))]
macro_rules! vfn {
    ($($signature:tt)*) => { unsafe extern "C" fn $($signature)* };
}

/// Defines free functions that the engine calls back *as* C++ virtual members.
///
/// Used for the vtables we fabricate ourselves (the game-event listener, the trace
/// filter), where the function bodies are ours but the ABI must match the engine's.
///
/// ```ignore
/// vfn_impl! {
///     fn get_debug_id(_this: *mut c_void) -> i32 { 42 }
/// }
/// ```
#[cfg(target_os = "windows")]
macro_rules! vfn_impl {
    ($(
        $(#[$meta:meta])*
        fn $name:ident($($arg:ident: $arg_ty:ty),* $(,)?) $(-> $ret:ty)? $body:block
    )*) => {$(
        $(#[$meta])*
        unsafe extern "thiscall" fn $name($($arg: $arg_ty),*) $(-> $ret)? $body
    )*};
}

#[cfg(not(target_os = "windows"))]
macro_rules! vfn_impl {
    ($(
        $(#[$meta:meta])*
        fn $name:ident($($arg:ident: $arg_ty:ty),* $(,)?) $(-> $ret:ty)? $body:block
    )*) => {$(
        $(#[$meta])*
        unsafe extern "C" fn $name($($arg: $arg_ty),*) $(-> $ret)? $body
    )*};
}

pub(crate) use {vfn, vfn_impl};
