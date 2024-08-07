use egui::Widget;
use log::debug;

use apex_framework::input::{
  keybinds::{Bind, KeyCombination},
  Input,
};

use crate::client::{
  action::ClientAction,
  settings::{Settings, SettingsProxy},
};

pub mod tab_controls;
pub mod tab_general;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSettingsTab {
  General,
  Controls,
}

pub struct GameSettingsView {
  pub tab: GameSettingsTab,
  pub is_open: bool,

  buffer: String,
  current_bind: Option<KeyCombination>,
  bind_cache: Vec<(KeyCombination, Bind<ClientAction>)>,
}

impl GameSettingsView {
  pub fn new() -> Self {
    return Self {
      tab: GameSettingsTab::General,
      is_open: false,

      buffer: String::new(),
      current_bind: None,
      bind_cache: vec![],
    };
  }

  pub fn prepare(
    &mut self,
    ctx: &egui::Context,
    input: &mut Input<ClientAction>,
    settings: &mut Settings,
    proxy: &mut impl SettingsProxy,
  ) {
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
      .show(ctx, |ui| {
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
          GameSettingsTab::General => self.general_tab(ui, settings, proxy),
          GameSettingsTab::Controls => self.controls_tab(ui, input),
        }
      });

    self.is_open = is_open;
  }
}
