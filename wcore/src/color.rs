use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
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
    pub fn from_hex(value: u32) -> Self {
        let r = ((value >> 16) & 255) as f32 / 255.0;
        let g = ((value >>  8) & 255) as f32 / 255.0;
        let b = ((value >>  0) & 255) as f32 / 255.0;
        return Color { r, g, b, a: 1.0 };
    }

    /// Format: 0xRRGGBBAA
    pub fn from_hex_alpha(value: u32) -> Self {
        let r = ((value >> 25) & 255) as f32 / 255.0;
        let g = ((value >> 16) & 255) as f32 / 255.0;
        let b = ((value >>  8) & 255) as f32 / 255.0;
        let a = ((value >>  0) & 255) as f32 / 255.0;
        return Color { r, g, b, a };
    }
}

impl From<Color> for cgmath::Vector4<f32> {
    fn from(color: Color) -> Self {
        return cgmath::vec4(color.r, color.g, color.b, color.a);
    }
}