use crate::{
  client::{client::Client, input::client_action::ClientAction, state::AppState, ui::game_settings::GameSettingsView},
  core::{core::Core, input::Input},
};

pub struct SettingsScreen {
  game_settings: GameSettingsView,
}

impl SettingsScreen {
  pub fn new() -> Self {
    let game_settings = GameSettingsView::new();

    return Self { game_settings };
  }

  pub fn prepare(&mut self, core: &Core<Client>, input: &mut Input<ClientAction>, state: &mut AppState) {
    self.game_settings.prepare(core, input, state);
  }

  pub fn is_settings_open(&self) -> bool {
    return self.game_settings.is_open;
  }

  pub fn toggle_settings(&mut self) {
    self.game_settings.is_open = !self.game_settings.is_open;
  }
}
