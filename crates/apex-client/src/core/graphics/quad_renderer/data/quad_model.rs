use bytemuck::{Pod, Zeroable};
use glam::{vec3, Mat4, Quat, Vec3};

use crate::core::graphics::{color::Color, instance::Instance};

#[derive(Copy, Clone)]
pub struct QuadModel {
  pub position : Vec3,
  pub scale    : Vec3,
  pub rotation : Quat,
  pub color    : Color,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct BakedModel {
  pub model: Mat4,
  pub color: Color,
}

impl QuadModel {
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

impl Instance for QuadModel {
  type Baked = BakedModel;

  fn bake(&self) -> Self::Baked {
    let model = Mat4::from_translation(self.position)
              * Mat4::from_scale(vec3(self.scale.x, self.scale.y, self.scale.z))
              * Mat4::from_rotation_translation(self.rotation, Vec3::ONE);

    return BakedModel { model, color: self.color };
  }
}
