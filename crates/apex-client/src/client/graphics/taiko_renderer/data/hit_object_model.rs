use apex_framework::{
  graphics::{color::Color, instance::Instance},
  time::time::Time,
};
use bytemuck::{Pod, Zeroable};
use glam::{vec3, Vec2, Vec3};

#[rustfmt::skip]
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct BakedHitObjectModel {
  pub size_offset : Vec3,
  pub velocity    : f32,
  pub color       : Vec3,
  pub finisher    : u32,
  pub hit         : f32,
}

#[rustfmt::skip]
pub struct HitObjectModel {
  pub time     : f32,
  pub size     : Vec2,
  pub color    : Color,
  pub finisher : bool,
  pub velocity : f32,
  pub hit      : Time,
}

impl HitObjectModel {
  const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
    2 => Float32x3,
    3 => Float32,
    4 => Float32x3,
    5 => Uint32,
    6 => Float32,
  ];

  pub fn describe() -> wgpu::VertexBufferLayout<'static> {
    use std::mem;
    wgpu::VertexBufferLayout {
      array_stride: mem::size_of::<BakedHitObjectModel>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Instance,
      attributes: &Self::ATTRIBS,
    }
  }
}

impl Instance for HitObjectModel {
  type Baked = BakedHitObjectModel;

  fn bake(&self) -> Self::Baked {
    #[rustfmt::skip]
    return BakedHitObjectModel {
      size_offset : vec3(self.size.x / self.velocity, self.size.y, self.time),
      velocity    : self.velocity,
      color       : self.color.into(),
      finisher    : if self.finisher { 1 } else { 0 },
      hit         : self.hit.to_seconds() as f32,
    };
  }
}
