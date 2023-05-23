use bytemuck::{Pod, Zeroable};
use cgmath::{Vector3, Quaternion, Matrix4};

use crate::{graphics::instance::Instance, color::Color};

#[derive(Copy, Clone)]
pub struct Model {
    pub position : Vector3<f32>,
    pub scale    : Vector3<f32>,
    pub rotation : Quaternion<f32>,
    pub color    : Color,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct BakedModel {
    pub model: Matrix4<f32>,
    pub color: Color,
}

impl Model {
    pub fn describe<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<BakedModel>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Instance for Model {
    type Baked = BakedModel;

    fn bake(&self) -> Self::Baked {
        let model = cgmath::Matrix4::from_translation(self.position)
                  * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
                  * cgmath::Matrix4::from(self.rotation);
        
        return BakedModel { model, color: self.color };
    }
}