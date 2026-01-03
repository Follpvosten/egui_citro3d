use std::collections::HashMap;

use citro3d::{math::FVec4, render::RenderPass};

use super::{TexAndData, configure_texenvs};
use crate::texdelta;

#[cfg(feature = "dbg_printlns")]
use ctru::prelude::{Hid, KeyPad};

pub(crate) struct RenderContext<'a, 'frame> {
    #[cfg(feature = "dbg_printlns")]
    pub hid: &'a Hid,
    pub pass: RenderPass<'frame>,
    pub ctx: &'a egui::Context,
    pub texmap: &'a mut HashMap<egui::TextureId, TexAndData>,
    pub projection_uniform_idx: citro3d::uniform::Index,
    pub attr_info: &'a citro3d::attrib::Info,
}

impl<'a, 'frame> RenderContext<'a, 'frame>
where
    'a: 'frame,
{
    pub(crate) fn render(
        &mut self,
        twovecs: [[f32; 4]; 2],
        render_target: &'a mut citro3d::render::Target<'_>,
        out: egui::FullOutput,
    ) {
        #[cfg(feature = "dbg_printlns")]
        {
            if !out.textures_delta.set.is_empty() {
                println!("Adding/Patching {} Textures", out.textures_delta.set.len());
            }
            if self.hid.keys_down().contains(KeyPad::B) {
                println!("Rendering {} shapes", out.shapes.len());
            }
            if self.hid.keys_down().contains(KeyPad::Y) {
                println!("{:#?}", out.shapes);
            }
        }

        texdelta::texdelta(self.texmap, out.textures_delta.set);
        let tessel = self.ctx.tessellate(out.shapes, 1.0);

        render_target.clear(citro3d::render::ClearFlags::ALL, 0xFF_00_00_00, 0);
        // let mut last_christmas_i_gave_you_my = None;

        self.pass
            .select_render_target(&*render_target)
            .expect("wharg");

        self.pass
            .bind_vertex_uniform(self.projection_uniform_idx, twovecs_to_uniform(twovecs));
        self.pass.set_attr_info(self.attr_info);
        #[cfg(feature = "dbg_printlns")]
        if self.hid.keys_down().contains(KeyPad::B) {
            println!("Rendering {} prims", tessel.len());
        }
        for t in tessel.into_iter() {
            let mesh = match t.primitive {
                egui::epaint::Primitive::Mesh(mesh) => mesh,
                egui::epaint::Primitive::Callback(_) => {
                    continue;
                }
            };
            let TexAndData { tex, data } = self.texmap.get_mut(&mesh.texture_id).unwrap();
            tex.bind(0);
            configure_texenvs::configure_texenv(&mut self.pass, data);
            for mesh in mesh.split_to_u16() {
                #[cfg(feature = "dbg_printlns")]
                if self.hid.keys_down().contains(KeyPad::X) {
                    println!("Tex  : {}x{}@{}", tex.width, tex.height, tex.format);
                    println!("Verts: ");
                    for vert in &mesh.vertices {
                        println!("{:?}", vert);
                    }
                    println!("Indices: ");
                    for arr in mesh.indices.chunks_exact(3) {
                        println!("({} {} {})", arr[0], arr[1], arr[2]);
                    }
                }
                use crate::cimm::attr;
                use crate::cimm::imm;
                imm(|| {
                    for i in mesh.indices {
                        let egui::epaint::Vertex { pos, uv, color } = mesh.vertices[i as usize];
                        attr([pos.x, pos.y, 0.0, 0.0]);
                        attr([uv.x, uv.y, 0.0, 0.0]);
                        attr([
                            color.r() as f32 / 255.0,
                            color.g() as f32 / 255.0,
                            color.b() as f32 / 255.0,
                            color.a() as f32 / 255.0,
                        ]);
                    }
                });
            }
            unsafe {
                use citro3d_sys::{C3D_DirtyTexEnv, C3D_GetTexEnv};
                let te = C3D_GetTexEnv(0);
                C3D_DirtyTexEnv(te);
            }
        }
        for remove in out.textures_delta.free {
            self.texmap.remove(&remove);
        }
    }
}

pub(crate) fn twovecs_to_uniform(twovecs_bottom: [[f32; 4]; 2]) -> citro3d::uniform::Uniform {
    citro3d::uniform::Uniform::Float2([
        FVec4::from_raw(citro3d_sys::C3D_FVec {
            c: twovecs_bottom[0],
        }),
        FVec4::from_raw(citro3d_sys::C3D_FVec {
            c: twovecs_bottom[1],
        }),
    ])
}
