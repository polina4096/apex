use crate::{client::{client::Client, event::ClientEvent, input::client_action::ClientAction, state::GameState, ui::game_settings::GameSettingsView}, core::{core::Core, event::EventBus, input::Input}};

pub struct SettingsScreen {
  game_settings: GameSettingsView,
}

impl SettingsScreen {
  pub fn new(event_bus: EventBus<ClientEvent>) -> Self {
    let game_settings = GameSettingsView::new(event_bus);

    return Self {
      game_settings,
    };
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, input: &mut Input<ClientAction>, state: &mut GameState) {
    self.game_settings.prepare(core.egui_ctx(), input, state);
  }

  pub fn is_settings_open(&self) -> bool {
    return self.game_settings.is_open;
  }

  pub fn toggle_settings(&mut self) {
    self.game_settings.is_open = !self.game_settings.is_open;
  }
}
