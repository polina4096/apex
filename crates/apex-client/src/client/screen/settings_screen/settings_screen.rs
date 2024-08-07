use apex_framework::input::Input;

use crate::client::{
  action::ClientAction,
  settings::{Settings, SettingsProxy},
  ui::game_settings::GameSettingsView,
};

pub struct SettingsScreen {
  game_settings: GameSettingsView,
}

impl SettingsScreen {
  pub fn new() -> Self {
    let game_settings = GameSettingsView::new();

    return Self { game_settings };
  }

  pub fn prepare(
    &mut self,
    ctx: &egui::Context,
    input: &mut Input<ClientAction>,
    settings: &mut Settings,
    proxy: &mut impl SettingsProxy,
  ) {
    self.game_settings.prepare(ctx, input, settings, proxy);
  }

  pub fn is_open(&self) -> bool {
    return self.game_settings.is_open;
  }

  pub fn toggle(&mut self) {
    self.game_settings.is_open = !self.game_settings.is_open;
  }
}
