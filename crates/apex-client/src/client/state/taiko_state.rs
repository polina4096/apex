use serde::{Deserialize, Serialize};

use crate::core::graphics::color::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaikoState {
  /// Hit object distance multiplier
  pub zoom: f64,

  /// Gameplay scale
  pub scale: f64,

  /// Hit position X
  pub hit_position_x: f32,

  /// Hit position Y
  pub hit_position_y: f32,

  /// Color of the don hit object
  pub don_color: Color,

  /// Color of the kat hit object
  pub kat_color: Color,

  /// Toggle hitsounds (TODO: implement proper volume control)
  pub hitsounds: bool,
}

impl Default for TaikoState {
  fn default() -> Self {
    return Self {
      zoom: 0.235,
      scale: 0.85,
      hit_position_x: 256.0,
      hit_position_y: 192.0,
      don_color: Color::new(0.92, 0.00, 0.27, 1.00),
      kat_color: Color::new(0.00, 0.47, 0.67, 1.00),
      hitsounds: true,
    };
  }
}
