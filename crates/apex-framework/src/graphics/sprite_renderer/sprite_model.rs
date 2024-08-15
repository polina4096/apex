use bytemuck::{Pod, Zeroable};
use glam::{vec2, vec3, Mat4, Quat, Vec2, Vec3};

use crate::graphics::{color::Color, instance::Instance, origin::Origin};

#[rustfmt::skip]
#[derive(Copy, Clone)]
pub struct SpriteModel {
  pub position  : Vec2,
  pub origin    : Origin,
  pub scale     : Vec2,
  pub rotation  : Quat,
  pub color     : Color,
  pub uv_offset : Vec2,
  pub uv_scale  : Vec2,
}

#[rustfmt::skip]
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct BakedSpriteModel {
  pub model     : Mat4,
  pub color     : Color,
  pub uv_offset : Vec2,
  pub uv_scale  : Vec2,
}

impl SpriteModel {
  pub fn describe<'a>() -> wgpu::VertexBufferLayout<'a> {
    wgpu::VertexBufferLayout {
      array_stride: std::mem::size_of::<BakedSpriteModel>() as wgpu::BufferAddress,
      step_mode: wgpu::VertexStepMode::Instance,
      attributes: &const {
        wgpu::vertex_attr_array![
          2 => Float32x4,
          3 => Float32x4,
          4 => Float32x4,
          5 => Float32x4,
          6 => Float32x4,
          7 => Float32x2,
          8 => Float32x2,
        ]
      },
    }
  }
}

impl Instance for SpriteModel {
  type Baked = BakedSpriteModel;

  fn bake(&self) -> Self::Baked {
    let scale = self.scale * 0.5;

    #[rustfmt::skip]
    let origin_offset = (match self.origin {
      Origin::TopLeft      => vec2(-0.5, -0.5),
      Origin::TopTop       => vec2( 0.0, -0.5),
      Origin::TopRight     => vec2( 0.5, -0.5),

      Origin::CenterLeft   => vec2(-0.5, 0.0),
      Origin::CenterCenter => vec2( 0.0, 0.0),
      Origin::CenterRight  => vec2( 0.5, 0.0),

      Origin::BottomLeft   => vec2(-0.5, 0.5),
      Origin::BottomBottom => vec2( 0.0, 0.5),
      Origin::BottomRight  => vec2( 0.5, 0.5),
    } - Vec2::splat(1.0)) * scale;

    #[rustfmt::skip] let model
      = Mat4::from_translation(self.position.extend(0.0))
      * Mat4::from_translation(origin_offset.extend(0.0))
      * Mat4::from_scale(vec3(scale.x, scale.y, 1.0))
      * Mat4::from_rotation_translation(self.rotation, Vec3::ONE);

    return BakedSpriteModel {
      model,
      color: self.color,
      uv_offset: self.uv_offset,
      uv_scale: self.uv_scale,
    };
  }
}
