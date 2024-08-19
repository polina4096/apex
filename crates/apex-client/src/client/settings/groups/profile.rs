use crate::client::settings::{proxy::ClientSettingsProxy, SettingsProxy};

use apex_framework::{SettingsGroup, SettingsSubgroup};
use macro_rules_attribute::derive;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

use crate::client::score::score::Score;

pub use super::super::ui as settings_ui;

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct ProfileSettingsGroup {
  #[custom(ui(name = "User"))]
  pub user: ProfileUserSettingsSubgroup,
}

#[derive(SettingsSubgroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct ProfileUserSettingsSubgroup {
  /// Player profile username
  #[default(String::from(Score::DEFAULT_USERNAME))]
  #[custom(ui(name = "Username", inline = true))]
  username: String,
}

impl ProfileUserSettingsSubgroupProxy for ClientSettingsProxy<'_, '_> {}
