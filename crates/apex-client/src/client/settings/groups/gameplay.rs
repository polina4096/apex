use crate::client::settings::{proxy::ClientSettingsProxy, SettingsProxy};

use apex_framework::{time::time::Time, SettingsGroup, SettingsSubgroup};
use macro_rules_attribute::derive;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

pub use super::super::ui as settings_ui;

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct GameplaySettingsGroup {
  #[custom(ui(name = "Audio"))]
  pub audio: GameplayAudioSettingsSubgroup,
}

#[derive(SettingsSubgroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct GameplayAudioSettingsSubgroup {
  /// Offset of the audio in milliseconds
  #[default = 0]
  #[custom(ui(name = "Universal Offset", range = -500 ..= 500))]
  universal_offset: i64,

  /// Additional time before the first note
  #[default = 1000]
  #[custom(ui(name = "Lead In", range = 0 ..= 5000))]
  lead_in: u64,

  /// Additional time after the last note
  #[default = 1000]
  #[custom(ui(name = "Lead out", range = 0 ..= 5000))]
  lead_out: u64,

  /// Additional time before a break overlay is show
  #[default = 1000]
  #[custom(ui(name = "Break Start Leniency", range = 0 ..= 5000))]
  break_leniency_start: u64,

  /// Break overlay is hidden this much earlier
  #[default = 1000]
  #[custom(ui(name = "Break End Leniency", range = 0 ..= 5000))]
  break_leniency_end: u64,
}

impl GameplayAudioSettingsSubgroupProxy for ClientSettingsProxy<'_, '_> {
  fn update_lead_in(&mut self, value: &u64) {
    self.audio.lead_in = Time::from_ms(*value as f64);
  }

  fn update_lead_out(&mut self, value: &u64) {
    self.audio.lead_out = Time::from_ms(*value as f64);
  }

  fn update_universal_offset(&mut self, value: &i64) {
    self.audio.audio_offset = Time::from_ms(*value as f64);
  }
}
