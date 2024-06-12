use glam::{vec2, vec3, Vec2, Vec3};

#[rustfmt::skip]
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadVertex {
  position : Vec3,
  uv       : Vec2,
}

impl QuadVertex {
  pub fn describe<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<QuadVertex>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Vertex,
      attributes: &[
        wgpu::VertexAttribute {
          offset: 0,
          shader_location: 0,
          format: wgpu::VertexFormat::Float32x3,
        },
        wgpu::VertexAttribute {
          offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
          shader_location: 1,
          format: wgpu::VertexFormat::Float32x2,
        },
      ],
    }
  }

  pub fn vertices_quad(min: f32, max: f32) -> Vec<Self> {
    #[rustfmt::skip] return vec![
      QuadVertex { position: vec3(min, min, 1.0), uv: vec2(0.0, 0.0) },
      QuadVertex { position: vec3(min, max, 1.0), uv: vec2(0.0, 1.0) },
      QuadVertex { position: vec3(max, max, 1.0), uv: vec2(1.0, 1.0) },
      QuadVertex { position: vec3(max, max, 1.0), uv: vec2(1.0, 1.0) },
      QuadVertex { position: vec3(max, min, 1.0), uv: vec2(1.0, 0.0) },
      QuadVertex { position: vec3(min, min, 1.0), uv: vec2(0.0, 0.0) },
    ];
  }
}
