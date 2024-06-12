use bytemuck::{Pod, Zeroable};
use glam::{vec3, vec4, Vec3, Vec4};
use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, Serialize, Deserialize)]
pub struct Color {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub a: f32,
}

impl Color {
  pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
    return Self { r, g, b, a };
  }

  pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let a = a as f32 / 255.0;
    return Self { r, g, b, a };
  }

  pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    return Self { r, g, b, a: 1.0 };
  }

  /// Format: 0xRRGGBB
  #[rustfmt::skip]
  #[allow(clippy::identity_op)]
  pub fn from_hex(value: u32) -> Self {
    let r = ((value >> 16) & 255) as f32 / 255.0;
    let g = ((value >>  8) & 255) as f32 / 255.0;
    let b = ((value >>  0) & 255) as f32 / 255.0;
    return Color { r, g, b, a: 1.0 };
  }

  /// Format: 0xRRGGBBAA
  #[rustfmt::skip]
  #[allow(clippy::identity_op)]
  pub fn from_hex_alpha(value: u32) -> Self {
    let r = ((value >> 25) & 255) as f32 / 255.0;
    let g = ((value >> 16) & 255) as f32 / 255.0;
    let b = ((value >>  8) & 255) as f32 / 255.0;
    let a = ((value >>  0) & 255) as f32 / 255.0;
    return Color { r, g, b, a };
  }

  pub fn as_rgba(&self) -> (f32, f32, f32, f32) {
    return (self.r, self.g, self.b, self.a);
  }
}

impl From<Color> for Vec4 {
  fn from(color: Color) -> Self {
    return vec4(color.r, color.g, color.b, color.a);
  }
}

impl From<Color> for Vec3 {
  fn from(color: Color) -> Self {
    return vec3(color.r, color.g, color.b);
  }
}

impl From<egui::Rgba> for Color {
  fn from(value: egui::Rgba) -> Self {
    return Color::new(value.r(), value.g(), value.b(), value.a());
  }
}
