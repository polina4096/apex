use egui::Widget;
use log::debug;
use tap::Tap;

use crate::{
  client::{client::Client, event::ClientEvent, input::client_action::ClientAction, state::AppState},
  core::{
    core::Core,
    event::EventBus,
    input::{
      bind::{Bind, KeyCombination},
      Input,
    },
  },
};

pub mod tab_controls;
pub mod tab_general;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSettingsTab {
  General,
  Controls,
}

pub struct GameSettingsView {
  event_bus: EventBus<ClientEvent>,

  pub tab: GameSettingsTab,
  pub is_open: bool,

  buffer: String,
  current_bind: Option<KeyCombination>,
  bind_cache: Vec<(KeyCombination, Bind<ClientAction>)>,
}

impl GameSettingsView {
  pub fn new(event_bus: EventBus<ClientEvent>) -> Self {
    return Self {
      event_bus,
      tab: GameSettingsTab::General,
      is_open: false,

      buffer: String::new(),
      current_bind: None,
      bind_cache: vec![],
    };
  }

  pub fn prepare(&mut self, core: &Core<Client>, input: &mut Input<ClientAction>, state: &mut AppState) {
    let mut is_open = self.is_open;

    // TODO: the cache won't be rebuilt if the keybinds are changed while the,
    // settings are open yet it doesn't matter right now as that is not possible.
    #[rustfmt::skip] if !is_open { self.bind_cache.clear(); return; };
    if self.bind_cache.is_empty() && !input.keybinds.is_empty() {
      debug!("Rebuilding bind cache");
      self.bind_cache = input.keybinds.as_vec();
    }

    egui::Window::new("Settings")
      .fixed_size(egui::vec2(384.0, 512.0))
      .resizable(false)
      .collapsible(false)
      .open(&mut is_open)
      .show(core.egui_ctx.egui_ctx(), |ui| {
        ui.horizontal(|ui| {
          let active = ui.style().visuals.widgets.active.bg_fill;
          let default = egui::Color32::TRANSPARENT;

          {
            let stroke = if self.tab == GameSettingsTab::General { active } else { default };
            let text = egui::RichText::new("General").strong().size(16.0);
            let button = egui::Button::new(text).fill(stroke);

            if button.ui(ui).clicked() {
              self.tab = GameSettingsTab::General;
            }
          }

          {
            let stroke = if self.tab == GameSettingsTab::Controls { active } else { default };
            let text = egui::RichText::new("Controls").strong().size(16.0);
            let button = egui::Button::new(text).fill(stroke);

            if button.ui(ui).clicked() {
              self.tab = GameSettingsTab::Controls;
            }
          }
        });

        ui.separator();

        match self.tab {
          GameSettingsTab::General => self.general_tab(ui, core, state),
          GameSettingsTab::Controls => self.controls_tab(ui, input),
        }
      });

    self.is_open = is_open;
  }
}
