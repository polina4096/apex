use apex_framework::{data::persistent::Persistent, SettingsStruct};
use groups::audio::AudioSettingsGroup;
use macro_rules_attribute::derive;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::client::settings::groups::{
  gameplay::GameplaySettingsGroup, graphics::GraphicsSettingsGroup, interface::InterfaceSettingsGroup,
  profile::ProfileSettingsGroup, taiko::TaikoSettingsGroup,
};

use crate::client::settings::groups::{
  audio::AudioSettingsGroupProxy, gameplay::GameplaySettingsGroupProxy, graphics::GraphicsSettingsGroupProxy,
  interface::InterfaceSettingsGroupProxy, profile::ProfileSettingsGroupProxy, taiko::TaikoSettingsGroupProxy,
};

pub mod groups;
pub mod proxy;
pub mod ui;

#[derive(SettingsStruct!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Settings {
  #[custom(ui(name = "Profile", icon = "👩"))]
  pub profile: ProfileSettingsGroup,

  #[custom(ui(name = "Audio", icon = "🔊"))]
  pub audio: AudioSettingsGroup,

  #[custom(ui(name = "Graphics", icon = "🖵"))]
  pub graphics: GraphicsSettingsGroup,

  #[custom(ui(name = "Interface", icon = "🗖"))]
  pub interface: InterfaceSettingsGroup,

  #[custom(ui(name = "Gameplay", icon = "🎮"))]
  pub gameplay: GameplaySettingsGroup,

  #[custom(ui(name = "Taiko", icon = "🎺"))]
  pub taiko: TaikoSettingsGroup,
}

impl Persistent for Settings {
  fn load(path: impl AsRef<std::path::Path>) -> Self {
    {
      let path = path.as_ref().canonicalize().unwrap_or(path.as_ref().to_owned());
      log::info!("Loading settings from `{}`", path.display());
    }

    return std::fs::read_to_string(&path)
      .map(|data| {
        return toml::from_str(&data).unwrap_or_else(|e| {
          log::error!("Failed to parse config file, falling back to default config: {}", e);

          return Settings::default();
        });
      })
      .unwrap_or_else(|e| {
        let default = Settings::default();

        match e.kind() {
          std::io::ErrorKind::NotFound => {
            log::warn!("Failed to open config file, file not found. Creating a default config file...");
            let default_data = toml::to_string_pretty(&default).expect("Failed to serialize default config");
            if let Err(e) = std::fs::write(&path, default_data) {
              log::error!("Failed to write default config file: {}", e);
            }
          }

          std::io::ErrorKind::PermissionDenied => {
            log::warn!("Failed to open config file, insufficient permissions. Falling back to default configuration.");
          }

          _ => {
            log::error!("Failed to access config file: {}. Falling back to default configuration.", e);
          }
        }

        return default;
      });
  }

  fn save(&self, path: impl AsRef<std::path::Path>) {
    let data = match toml::to_string_pretty(&self) {
      Ok(data) => data,
      Err(e) => {
        log::error!("Failed to serialize settings: {}", e);
        return;
      }
    };

    if let Err(e) = std::fs::write(&path, data) {
      log::error!("Failed to write settings to file: {}", e);
      return;
    }

    let path = path.as_ref().canonicalize().unwrap_or(path.as_ref().to_owned());
    log::info!("Settings successfully written to `{}`", path.display());
  }
}
