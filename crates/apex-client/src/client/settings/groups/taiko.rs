use crate::client::settings::{proxy::ClientSettingsProxy, SettingsProxy};

use apex_framework::{graphics::color::Color, SettingsGroup, SettingsSubgroup};
use macro_rules_attribute::derive;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

pub use super::super::ui as settings_ui;

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct TaikoSettingsGroup {
  #[custom(ui(name = "General"))]
  pub general: TaikoGeneralSettingsSubgroup,
}

#[derive(SettingsSubgroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct TaikoGeneralSettingsSubgroup {
  /// Hit object distance multiplier
  #[default = 0.215]
  #[custom(ui(name = "Conveyor Zoom", range = 0.0 ..= 2.0))]
  conveyor_zoom: f64,

  /// Gameplay scale
  #[default = 0.85]
  #[custom(ui(name = "Gameplay Scale", range = 0.0 ..= 10.0))]
  gameplay_scale: f64,

  /// Hit position X
  #[default = 256.0]
  #[custom(ui(name = "Horizontal Position", clamp = false, slider = false))]
  hit_position_x_px: f32,

  /// Hit position Y
  #[default = 0.35]
  #[custom(ui(name = "Vertical Position", range = 0.0 ..= 1.0))]
  hit_position_y_perc: f32,

  /// Color of the don hit object
  #[default(Color::new(0.92, 0.00, 0.27, 1.00))]
  #[custom(ui(name = "Don Color"))]
  don_color: Color,

  /// Color of the kat hit object
  #[default(Color::new(0.00, 0.47, 0.67, 1.00))]
  #[custom(ui(name = "Kat Color"))]
  kat_color: Color,

  /// Hit animation
  #[default = true]
  #[custom(ui(name = "Hit Animation"))]
  hit_animation: bool,
}

impl TaikoGeneralSettingsSubgroupProxy for ClientSettingsProxy<'_, '_> {
  fn update_conveyor_zoom(&mut self, value: &f64) {
    self.gameplay_screen.set_conveyor_zoom(self.device, self.queue, *value);
  }

  fn update_gameplay_scale(&mut self, value: &f64) {
    self.gameplay_screen.set_gameplay_scale(self.device, self.queue, *value);
  }

  fn update_hit_position_x_px(&mut self, value: &f32) {
    self.gameplay_screen.set_hit_position_x_px(self.device, self.queue, *value);
  }

  fn update_hit_position_y_perc(&mut self, value: &f32) {
    self.gameplay_screen.set_hit_position_y_perc(self.device, self.queue, *value);
  }

  fn update_don_color(&mut self, value: &Color) {
    self.gameplay_screen.set_don_color(self.device, *value);
  }

  fn update_kat_color(&mut self, value: &Color) {
    self.gameplay_screen.set_kat_color(self.device, *value);
  }

  fn update_hit_animation(&mut self, value: &bool) {
    self.gameplay_screen.set_hit_animation_height(
      self.device,
      self.config.format,
      // Apparently setting it to f64::INFINITY leads to a crash, see https://github.com/gfx-rs/wgpu/issues/6082
      if *value { 12.5 } else { 9999999.0 },
    );
  }
}
