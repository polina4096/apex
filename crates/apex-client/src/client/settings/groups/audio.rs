use crate::client::settings::{proxy::ClientSettingsProxy, SettingsProxy};

use apex_framework::{SettingsGroup, SettingsSubgroup};
use macro_rules_attribute::derive;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

pub use super::super::ui as settings_ui;

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct AudioSettingsGroup {
  #[custom(ui(name = "Volume"))]
  pub volume: AudioVolumeSettingsSubgroup,
}

#[derive(SettingsSubgroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct AudioVolumeSettingsSubgroup {
  /// Master volume
  #[default = 0.25]
  #[custom(ui(name = "Master Volume", range = 0.0 ..= 1.0, percentage = true, inline = true))]
  master_volume: f32,

  /// Music volume
  #[default = 1.0]
  #[custom(ui(name = "Music Volume", range = 0.0 ..= 1.0, percentage = true, inline = true))]
  music_volume: f32,

  /// Effect volume
  #[default = 1.0]
  #[custom(ui(name = "Effects Volume", range = 0.0 ..= 1.0, percentage = true, inline = true))]
  effects_volume: f32,
}

impl AudioVolumeSettingsSubgroupProxy for ClientSettingsProxy<'_, '_> {
  fn update_master_volume(&mut self, value: &f32) {
    self.audio.set_master_volume(*value);
  }

  fn update_music_volume(&mut self, value: &f32) {
    self.audio.set_music_volume(*value);
  }

  fn update_effects_volume(&mut self, value: &f32) {
    self.audio.set_effect_volume(*value);
  }
}
