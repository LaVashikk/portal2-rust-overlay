//! d3d9_hook_core: Shared D3D9 hooking core for all containers.
//!
//! Responsibilities:
//! - Patch IDirect3D9::CreateDevice (method 1) vtable entry (index 16).
//! - Patch IDirect3DDevice9::Reset (index 16) and Present (index 17).
//! - Expose two entrypoints:
//!   - install_on_d3d9(d3d9_ptr, callbacks) — proxy scenario (method 1).
//!   - start_offsets_hook_thread(offsets, delay_ms, callbacks) — client-wrapper/server-plugin (methods 2/3).
//! - Invoke callbacks for overlay lifecycle: on_device_created / on_present / on_pre_reset / on_post_reset.

#![cfg(all(target_os = "windows", target_pointer_width = "32"))]

use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};
use std::thread;

use windows::core::HRESULT;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Direct3D9::{
    D3DDEVICE_CREATION_PARAMETERS, D3DPRESENT_PARAMETERS, IDirect3D9, IDirect3DDevice9,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS};

type FnPresent = unsafe extern "system" fn(
    this: IDirect3DDevice9,
    src: *const RECT,
    dst: *const RECT,
    hwnd: HWND,
    dirty: *const c_void,
) -> HRESULT;

type FnReset = unsafe extern "system" fn(
    this: IDirect3DDevice9,
    params: *mut D3DPRESENT_PARAMETERS,
) -> HRESULT;

type FnCreateDevice = unsafe extern "system" fn(
    this: IDirect3D9,
    adapter: u32,
    devicetype: u32,
    hfocuswindow: HWND,
    behaviorflags: u32,
    ppresentationparameters: *mut D3DPRESENT_PARAMETERS,
    ppreturneddeviceinterface: *mut Option<IDirect3DDevice9>,
) -> HRESULT;

/// User-provided callbacks fired by the hook.
pub struct Callbacks {
    pub on_device_created: fn(HWND, &IDirect3DDevice9),
    pub on_pre_reset: fn(),
    pub on_post_reset: fn(&IDirect3DDevice9),
    pub on_present: fn(&IDirect3DDevice9),
}

struct HookState {
    callbacks: Option<&'static Callbacks>,

    device: Option<IDirect3DDevice9>,
    hwnd: Option<HWND>,

    // Original function pointers
    o_create_device: Option<FnCreateDevice>,
    o_present: Option<FnPresent>,
    o_reset: Option<FnReset>,

    // Flags
    present_installed: bool,
    reset_installed: bool,
    device_notified: bool,
}

unsafe impl Send for HookState {}
unsafe impl Sync for HookState {}

static STATE: OnceLock<Mutex<HookState>> = OnceLock::new();

fn state() -> &'static Mutex<HookState> {
    STATE.get_or_init(|| {
        Mutex::new(HookState {
            callbacks: None,
            device: None,
            hwnd: None,
            o_create_device: None,
            o_present: None,
            o_reset: None,
            present_installed: false,
            reset_installed: false,
            device_notified: false,
        })
    })
}

/// Install CreateDevice hook on an IDirect3D9 instance (method 1 / d3d9 proxy).
///
/// Safety: Caller must ensure `d3d9` is a valid IDirect3D9 object pointer from Direct3DCreate9.
pub unsafe fn install_on_d3d9(d3d9: *mut IDirect3D9, cb: &'static Callbacks) -> anyhow::Result<()> { unsafe {
    {
        let mut st = state().lock().unwrap();
        st.callbacks = Some(cb);
    }

    // IDirect3D9::CreateDevice is at vtable index 16.
    let vtable_ptr = *(d3d9 as *mut *mut usize);
    let create_device_fn_ptr_location = vtable_ptr.add(16);

    {
        let mut st = state().lock().unwrap();
        if st.o_create_device.is_none() {
            st.o_create_device = Some(std::mem::transmute(create_device_fn_ptr_location.read()));
        }
    }

    // Patch vtable entry to our hook.
    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    VirtualProtect(
        create_device_fn_ptr_location as _,
        std::mem::size_of::<usize>(),
        PAGE_EXECUTE_READWRITE,
        &mut old_protect,
    )
    .ok()
    .expect("VirtualProtect failed for CreateDevice entry");

    create_device_fn_ptr_location.write(hooked_create_device as usize);

    VirtualProtect(
        create_device_fn_ptr_location as _,
        std::mem::size_of::<usize>(),
        old_protect,
        &mut old_protect,
    )
    .ok()
    .expect("VirtualProtect restore failed for CreateDevice entry");

    Ok(())
}}

/// Start a thread that tries to find the device via offsets (methods 2/3).
pub fn start_offsets_hook_thread(
    offsets: &'static [usize],
    delay_ms: u64,
    cb: &'static Callbacks,
) {
    {
        let mut st = state().lock().unwrap();
        st.callbacks = Some(cb);
    }

    thread::spawn(move || {
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }

        unsafe {
            if let Some(device) = get_d3d_device_by_offsets(offsets) {
                install_device_hooks(&device);

                // Resolve HWND from creation parameters
                let mut params = D3DDEVICE_CREATION_PARAMETERS::default();
                if device.GetCreationParameters(&mut params).is_ok() {
                    let hwnd = params.hFocusWindow;
                    let mut st = state().lock().unwrap();
                    st.device = Some(device.clone());
                    st.hwnd = Some(hwnd);

                    if !st.device_notified {
                        if let Some(cb) = st.callbacks {
                            log::debug!("Offset-hook notifying device created: {:?}, {:?}", hwnd, device);
                            (cb.on_device_created)(hwnd, &device);
                        }
                        st.device_notified = true;
                    }
                } else {
                    log::warn!("Offset-hook GetCreationParameters failed, HWND may be missing.");
                }
            } else {
                log::error!("Offsets-based device acquisition failed.");
            }
        }
    });
}

// ============================================================
// Hooked functions
// ============================================================

unsafe extern "system" fn hooked_create_device(
    this: IDirect3D9,
    adapter: u32,
    devicetype: u32,
    hfocuswindow: HWND,
    behaviorflags: u32,
    ppresentationparameters: *mut D3DPRESENT_PARAMETERS,
    ppreturneddeviceinterface: *mut Option<IDirect3DDevice9>,
) -> HRESULT { unsafe {
    // Call original CreateDevice first.
    let hr = {
        let st = state().lock().unwrap();
        (st.o_create_device.unwrap())(
            this,
            adapter,
            devicetype,
            hfocuswindow,
            behaviorflags,
            ppresentationparameters,
            ppreturneddeviceinterface,
        )
    };

    if hr.is_ok() {
        if let Some(Some(device)) = ppreturneddeviceinterface.as_mut() {
            // Notify overlay first.
            {
                let mut st = state().lock().unwrap();
                st.device = Some(device.clone());
                st.hwnd = Some(hfocuswindow);

                if !st.device_notified {
                    if let Some(cb) = st.callbacks {
                        (cb.on_device_created)(hfocuswindow, device);
                    }
                    st.device_notified = true;
                }
            }

            // Then install Present/Reset hooks on the device.
            install_device_hooks(device);
        }
    }

    hr
}}

unsafe extern "system" fn hooked_reset(
    this: IDirect3DDevice9,
    params: *mut D3DPRESENT_PARAMETERS,
) -> HRESULT { unsafe {
    // Pre-reset callback
    {
        let st = state().lock().unwrap();
        if let Some(cb) = st.callbacks {
            (cb.on_pre_reset)();
        }
    }

    // Call original Reset
    let hr = {
        let st = state().lock().unwrap();
        (st.o_reset.unwrap())(this.clone(), params)
    };

    if hr.is_ok() {
        // Post-reset callback
        {
            let st = state().lock().unwrap();
            if let Some(cb) = st.callbacks {
                (cb.on_post_reset)(&this);
            }
        }
    }

    hr
}}

unsafe extern "system" fn hooked_present(
    this: IDirect3DDevice9,
    src: *const RECT,
    dst: *const RECT,
    hwnd: HWND,
    dirty: *const c_void,
) -> HRESULT { unsafe {
    // Lazy on_device_created if we didn't notify yet (covers offsets path).
    {
        let mut st = state().lock().unwrap();
        if !st.device_notified {
            // Try resolve HWND from device if not present
            let hwnd_to_use = if let Some(h) = st.hwnd {
                h
            } else {
                let mut params = D3DDEVICE_CREATION_PARAMETERS::default();
                if this.GetCreationParameters(&mut params).is_ok() {
                    params.hFocusWindow
                } else {
                    hwnd // fallback
                }
            };
            st.hwnd = Some(hwnd_to_use);
            st.device = Some(this.clone());

            if let Some(cb) = st.callbacks {
                (cb.on_device_created)(hwnd_to_use, &this);
            }
            st.device_notified = true;
        }
    }

    // Per-frame callback
    {
        let st = state().lock().unwrap();
        if let Some(cb) = st.callbacks {
            (cb.on_present)(&this);
        }
    }

    // Call original Present
    let hr = {
        let st = state().lock().unwrap();
        (st.o_present.unwrap())(this, src, dst, hwnd, dirty)
    };

    hr
}}

// ============================================================
// Device hooks installation
// ============================================================

unsafe fn install_device_hooks(device: &IDirect3DDevice9) { unsafe {
    let mut st = state().lock().unwrap();
    if st.present_installed && st.reset_installed {
        return;
    }

    // Get COM object vtable
    let com_ptr = *(device as *const _ as *const usize);
    let vtable_ptr = *(com_ptr as *const usize);

    // Reset is vtable index 16
    let reset_entry = (vtable_ptr + 16 * std::mem::size_of::<usize>()) as *mut usize;
    if !st.reset_installed {
        st.o_reset = Some(std::mem::transmute(reset_entry.read()));
        patch_vtable_entry(reset_entry, hooked_reset as usize);
        st.reset_installed = true;
    }

    // Present is vtable index 17
    let present_entry = (vtable_ptr + 17 * std::mem::size_of::<usize>()) as *mut usize;
    if !st.present_installed {
        st.o_present = Some(std::mem::transmute(present_entry.read()));
        patch_vtable_entry(present_entry, hooked_present as usize);
        st.present_installed = true;
    }
}}

unsafe fn patch_vtable_entry(entry: *mut usize, new_fn: usize) { unsafe {
    let mut old_protect = PAGE_PROTECTION_FLAGS(0);
    VirtualProtect(
        entry as _,
        std::mem::size_of::<usize>(),
        PAGE_EXECUTE_READWRITE,
        &mut old_protect,
    )
    .ok()
    .expect("VirtualProtect failed for vtable entry");

    entry.write(new_fn);

    VirtualProtect(
        entry as _,
        std::mem::size_of::<usize>(),
        old_protect,
        &mut old_protect,
    )
    .ok()
    .expect("VirtualProtect restore failed for vtable entry");
}}

/// Uninstalls device hooks (Present/Reset) by restoring original vtable entries.
/// Safe to call multiple times. Returns true if anything was restored.
pub fn uninstall() -> bool {
    unsafe {
        let mut st = state().lock().unwrap();

        // If we have no device, we can't restore vtable entries safely.
        let Some(device) = st.device.clone() else {
            return false;
        };

        // Recompute vtable pointers from the device we hooked.
        let com_ptr = *(&device as *const _ as *const usize);
        if com_ptr == 0 {
            return false;
        }
        let vtable_ptr = *(com_ptr as *const usize);
        if vtable_ptr == 0 {
            return false;
        }

        let mut restored_any = false;

        // Reset (index 16)
        if st.reset_installed {
            if let Some(orig) = st.o_reset {
                let entry = (vtable_ptr + 16 * std::mem::size_of::<usize>()) as *mut usize;
                patch_vtable_entry(entry, orig as usize);
                st.reset_installed = false;
                restored_any = true;
            }
        }

        // Present (index 17)
        if st.present_installed {
            if let Some(orig) = st.o_present {
                let entry = (vtable_ptr + 17 * std::mem::size_of::<usize>()) as *mut usize;
                patch_vtable_entry(entry, orig as usize);
                st.present_installed = false;
                restored_any = true;
            }
        }

        // Optionally clear callbacks and device references.
        st.callbacks = None;
        st.device = None;
        st.hwnd = None;
        st.device_notified = false;

        restored_any
    }
}

// Hardcoded renderer modules to scan for the D3D9 device.
const SHADER_API_MODULES: &[&str] = &["shaderapidx9.dll", "shaderapivk.dll"];

// Offsets-based device acquisition
unsafe fn get_d3d_device_by_offsets(offsets: &[usize]) -> Option<IDirect3DDevice9> { unsafe {
    for &module in SHADER_API_MODULES {
        log::debug!("Attempting to get module handle for {}...", module);
        let mut module_cstr = Vec::with_capacity(module.len() + 1);
        module_cstr.extend_from_slice(module.as_bytes());
        module_cstr.push(0);
        let shader_api = match GetModuleHandleA(windows::core::PCSTR(module_cstr.as_ptr())) {
            Ok(h) if !h.is_invalid() => h,
            _ => {
                log::debug!("{} not found in process, skipping.", module);
                continue; // Try the next module
            }
        };
        let base_addr = shader_api.0 as usize;
        log::debug!("{} found at base: 0x{:X}", module, base_addr);

        for (idx, &offset) in offsets.iter().enumerate() {
            let device_ptr_addr = base_addr + offset;
            log::debug!(
                "Trying offset #{} (0x{:X}) -> addr 0x{:X}",
                idx + 1,
                offset,
                device_ptr_addr
            );

            let device_ptr = *(device_ptr_addr as *const usize);
            log::debug!("Device pointer: 0x{:X}", device_ptr);

            if device_ptr == 0 || device_ptr < 0x10000 || device_ptr > 0x7FFFFFFF {
                log::debug!("Invalid pointer value");
                continue;
            }

            let vtable_ptr = *(device_ptr as *const usize);
            log::debug!("VTable: 0x{:X}", vtable_ptr);

            if vtable_ptr == 0 || vtable_ptr < 0x10000 || vtable_ptr > 0x7FFFFFFF {
                log::debug!("!! Invalid vtable !!");
                continue;
            }

            let present_addr = *((vtable_ptr + 17 * 4) as *const usize);
            log::debug!("  Present function: 0x{:X}", present_addr);

            if present_addr == 0 || present_addr < 0x10000 {
                log::debug!("  Invalid Present address");
                continue;
            }

            let device: IDirect3DDevice9 = std::mem::transmute(device_ptr as *mut c_void);

            log::info!("D3D9 hook successfully initialized via offsets.");

            std::mem::forget(device.clone());
            return Some(device);
        }
    }

    log::error!("Failed to initialize graphics hook. The overlay will not work.");
    None
}}
