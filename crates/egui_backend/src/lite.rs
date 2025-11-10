//! EguiDx9Lite: slim D3D9 backend for egui without storing user state/generic T.
//!
//! - No user data stored inside the backend.
//! - WndProc is NOT installed here; external code calls wnd_proc to feed input.
//! - `present` takes a draw closure to render UI.

#![cfg(all(target_os = "windows", target_pointer_width = "32"))]

use egui::{Context, epaint::Primitive};
use windows::Win32::{
    Foundation::{HWND, LPARAM, RECT, WPARAM},
    Graphics::Direct3D9::{IDirect3DDevice9, D3DPT_TRIANGLELIST, D3DVIEWPORT9},
    UI::WindowsAndMessaging::GetClientRect,
};

use crate::{
    inputman::InputManager,
    mesh::{Buffers, GpuVertex, MeshDescriptor},
    state::DxState,
    texman::TextureManager,
};

pub struct EguiDx9Lite {
    hwnd: HWND,
    reactive: bool,
    input_man: InputManager,
    tex_man: TextureManager,
    ctx: Context,
    buffers: Buffers,
    prims: Vec<MeshDescriptor>,
    last_idx_capacity: usize,
    last_vtx_capacity: usize,
}

unsafe impl Send for EguiDx9Lite {}

impl EguiDx9Lite {
    pub fn init(dev: &IDirect3DDevice9, hwnd: HWND, reactive: bool) -> Self {
        if hwnd.is_invalid() {
            panic!("invalid hwnd specified in egui init");
        }

        Self {
            hwnd,
            reactive,
            tex_man: TextureManager::new(),
            input_man: InputManager::new(hwnd),
            ctx: Context::default(),
            buffers: Buffers::create_buffers(dev, 16384, 16384),
            prims: Vec::new(),
            last_idx_capacity: 0,
            last_vtx_capacity: 0,
        }
    }

    pub fn pre_reset(&mut self) {
        self.buffers.delete_buffers();
        self.tex_man.deallocate_textures();
    }

    pub fn post_reset(&mut self, dev: &IDirect3DDevice9) {
        self.ctx = Context::default();
        self.buffers = Buffers::create_buffers(dev, 16384, 16384);
        self.tex_man.reallocate_textures(dev);
        self.ctx.request_repaint();
    }

    pub fn present(&mut self, dev: &IDirect3DDevice9, mut draw: impl FnMut(&Context)) {
        if unsafe { dev.TestCooperativeLevel() }.is_err() {
            return;
        }

        let output = self.ctx.run(self.input_man.collect_input(), |ctx| {
            draw(ctx);
        });

        if !output.textures_delta.is_empty() {
            self.tex_man.process_set_deltas(dev, &output.textures_delta);
        }

        if output.shapes.is_empty() {
            if !output.textures_delta.is_empty() {
                self.tex_man.process_free_deltas(&output.textures_delta);
            }
            return;
        }

        if self.ctx.has_requested_repaint() || !self.reactive {
            let mut vertices: Vec<GpuVertex> = Vec::with_capacity(self.last_vtx_capacity + 512);
            let mut indices: Vec<u32> = Vec::with_capacity(self.last_idx_capacity + 512);

            self.prims = self
                .ctx
                .tessellate(output.shapes, output.pixels_per_point)
                .into_iter()
                .filter_map(|prim| {
                    if let Primitive::Mesh(mesh) = prim.primitive {
                        if let Some((gpumesh, verts, idxs)) =
                            MeshDescriptor::from_mesh(mesh, prim.clip_rect)
                        {
                            vertices.extend_from_slice(verts.as_slice());
                            indices.extend_from_slice(idxs.as_slice());
                            Some(gpumesh)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            self.last_vtx_capacity = vertices.len();
            self.last_idx_capacity = indices.len();

            self.buffers.update_vertex_buffer(dev, &vertices);
            self.buffers.update_index_buffer(dev, &indices);
        }

        let _state = DxState::setup(dev, self.get_viewport());

        unsafe {
            let vtx = self.buffers.vtx.as_ref().expect("vertex buffer missing");
            dev.SetStreamSource(0, vtx, 0, std::mem::size_of::<GpuVertex>() as _)
                .expect("SetStreamSource failed");

            let idx = self.buffers.idx.as_ref().expect("index buffer missing");
            dev.SetIndices(idx).expect("SetIndices failed");
        }

        let mut our_vtx_idx: usize = 0;
        let mut our_idx_idx: usize = 0;

        self.prims.iter().for_each(|mesh: &MeshDescriptor| unsafe {
            dev.SetScissorRect(&mesh.clip).expect("SetScissorRect failed");

            let texture = self.tex_man.get_by_id(mesh.texture_id);
            dev.SetTexture(0, texture).expect("SetTexture failed");

            dev.DrawIndexedPrimitive(
                D3DPT_TRIANGLELIST,
                our_vtx_idx as _,
                0,
                mesh.vertices as _,
                our_idx_idx as _,
                (mesh.indices / 3usize) as _,
            )
            .expect("DrawIndexedPrimitive failed");

            our_vtx_idx += mesh.vertices;
            our_idx_idx += mesh.indices;
        });

        if !output.textures_delta.is_empty() {
            self.tex_man.process_free_deltas(&output.textures_delta);
        }
    }

    #[inline]
    pub fn wnd_proc(&mut self, umsg: u32, wparam: WPARAM, lparam: LPARAM) {
        self.input_man.process(umsg, wparam.0, lparam.0);
    }

    fn get_screen_size(&self) -> (f32, f32) {
        let mut rect = RECT::default();
        unsafe {
            GetClientRect(self.hwnd, &mut rect).expect("GetClientRect failed");
        }
        ((rect.right - rect.left) as f32, (rect.bottom - rect.top) as f32)
    }

    fn get_viewport(&self) -> D3DVIEWPORT9 {
        let (w, h) = self.get_screen_size();
        D3DVIEWPORT9 {
            X: 0,
            Y: 0,
            Width: w as _,
            Height: h as _,
            MinZ: 0.,
            MaxZ: 1.,
        }
    }
}
