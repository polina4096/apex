use bytemuck::{Pod, Zeroable};
use cgmath::{Vector4, Vector3, vec3};
use wcore::{graphics::instance::Instance, color::Color};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct BakedTaikoHitObjectModel {
    pub size_offset : Vector3<f32>,
    pub velocity    : f32,
    pub color       : Vector4<f32>,
    pub finisher    : u32,
}

pub struct TaikoHitObjectModel {
    pub time     : f32,
    pub size     : cgmath::Vector2<f32>,
    pub color    : Color,
    pub finisher : bool,
    pub velocity : f32,
}

impl TaikoHitObjectModel {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        2 => Float32x3,
        3 => Float32,
        4 => Float32x4,
        5 => Uint32,
    ];

    pub fn describe() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<BakedTaikoHitObjectModel>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS
        }
    }
}

impl Instance for TaikoHitObjectModel {
    type Baked = BakedTaikoHitObjectModel;

    fn bake(&self) -> Self::Baked {
        return BakedTaikoHitObjectModel {
            size_offset : vec3(self.size.x / self.velocity, self.size.y, self.time),
            velocity    : self.velocity,
            color       : self.color.into(),
            finisher    : if self.finisher { 1 } else { 0 },
        };
    }
}