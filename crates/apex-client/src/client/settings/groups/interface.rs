use crate::client::settings::{proxy::ClientSettingsProxy, SettingsProxy};

use apex_framework::{graphics::drawable::Drawable as _, time::time::Time, SettingsGroup, SettingsSubgroup};
use macro_rules_attribute::derive;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

pub use super::super::ui as settings_ui;

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct InterfaceSettingsGroup {
  #[custom(ui(name = "Delta Bar"))]
  pub delta_bar: InterfaceDeltaBarSettingsSubgroup,

  #[custom(ui(name = "Gameplay"))]
  pub gameplay: InterfaceGameplaySettingsSubgroup,
}

#[derive(SettingsSubgroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct InterfaceDeltaBarSettingsSubgroup {
  /// Total width of the hit delta bar
  #[default = 128.0]
  #[custom(ui(name = "Bar Width", clamp = false, slider = false))]
  delta_bar_width: f32,

  /// Height of the hit delta bar mark
  #[default = 24.0]
  #[custom(ui(name = "Bar Height", clamp = false, slider = false))]
  delta_bar_height: f32,

  /// Opacity of the hit delta bar areas
  #[default = 0.05]
  #[custom(ui(name = "Bar Opacity", range = 0.0 ..= 1.0))]
  delta_bar_opacity: f32,

  /// Width of the hit delta marker
  #[default = 2.0]
  #[custom(ui(name = "Marker Width", clamp = false, slider = false))]
  delta_marker_width: f32,

  /// Height of the hit delta marker
  #[default = 16.0]
  #[custom(ui(name = "Marker Height", clamp = false, slider = false))]
  delta_marker_height: f32,

  /// Opacity of the hit delta marker
  #[default = 0.25]
  #[custom(ui(name = "Marker Opacity", range = 0.0 ..= 1.0))]
  delta_marker_opacity: f32,

  /// Duration for which the hit delta marker is shown in seconds
  #[default = 1.0]
  #[custom(ui(name = "Hit Duration", range = 0.0 ..= 10.0))]
  delta_marker_duration: f32,

  /// Duration it takes for the hit delta marker to fade in or out in seconds
  #[default = 0.2]
  #[custom(ui(name = "Fade Duration", range = 0.0 ..= 10.0))]
  delta_marker_fade: f32,
}

#[derive(SettingsSubgroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct InterfaceGameplaySettingsSubgroup {
  #[default = false]
  #[custom(ui(name = "Letterboxing"))]
  letterboxing: bool,

  #[default = 1.0]
  #[custom(ui(name = "Width", range = 0.0 ..= 1.0, percentage = true, inline = true))]
  gameplay_width: f32,

  #[default = 1.0]
  #[custom(ui(name = "Height", range = 0.0 ..= 1.0, percentage = true, inline = true))]
  gameplay_height: f32,
}

impl InterfaceDeltaBarSettingsSubgroupProxy for ClientSettingsProxy<'_, '_> {
  fn update_delta_bar_width(&mut self, value: &f32) {
    self.gameplay_screen.set_delta_bar_width(*value);
  }

  fn update_delta_bar_height(&mut self, value: &f32) {
    self.gameplay_screen.set_delta_bar_height(*value);
  }

  fn update_delta_bar_opacity(&mut self, value: &f32) {
    self.gameplay_screen.set_delta_bar_opacity(*value);
  }

  fn update_delta_marker_width(&mut self, value: &f32) {
    self.gameplay_screen.set_delta_marker_width(*value);
  }

  fn update_delta_marker_height(&mut self, value: &f32) {
    self.gameplay_screen.set_delta_marker_height(*value);
  }

  fn update_delta_marker_opacity(&mut self, value: &f32) {
    self.gameplay_screen.set_delta_marker_opacity(*value);
  }

  fn update_delta_marker_duration(&mut self, value: &f32) {
    self.gameplay_screen.set_delta_marker_duration(Time::from_seconds(*value));
  }

  fn update_delta_marker_fade(&mut self, value: &f32) {
    self.gameplay_screen.set_delta_marker_fade(Time::from_seconds(*value));
  }
}

impl InterfaceGameplaySettingsSubgroupProxy for ClientSettingsProxy<'_, '_> {
  fn update_gameplay_width(&mut self, value: &f32) {
    self.backbuffer.set_scale_x(self.queue, *value);
    self.gameplay_screen.resize_width(self.device, self.queue, self.width * *value)
  }

  fn update_gameplay_height(&mut self, value: &f32) {
    self.backbuffer.set_scale_y(self.queue, *value);
    self.gameplay_screen.resize_height(self.device, self.queue, self.height * *value)
  }
}
