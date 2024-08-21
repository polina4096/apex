use std::fmt::Write as _;

use egui::Widget as _;

use apex_framework::input::{keybinds::KeyCombination, Input};

use crate::client::action::ClientAction;

use super::GameSettingsView;

impl GameSettingsView {
  pub(super) fn controls_tab(&mut self, ui: &mut egui::Ui, input: &mut Input<ClientAction>) {
    egui_extras::StripBuilder::new(ui)
      .size(egui_extras::Size::exact(64.0))
      .size(egui_extras::Size::exact(200.0))
      .size(egui_extras::Size::exact(128.0))
      .horizontal(|mut strip| {
        strip.cell(|ui| {
          ui.strong("Action");
        });

        strip.cell(|ui| {
          ui.strong("Description");
        });

        strip.cell(|ui| {
          ui.strong("Keybind");
        });
      });

    for (comb, bind) in &self.bind_cache {
      egui_extras::StripBuilder::new(ui)
        .size(egui_extras::Size::exact(64.0))
        .size(egui_extras::Size::exact(200.0))
        .size(egui_extras::Size::exact(128.0))
        .horizontal(|mut strip| {
          strip.cell(|ui| {
            ui.label(&bind.name);
          });

          strip.cell(|ui| {
            ui.label(&bind.description);
          });

          strip.cell(|ui| {
            self.buffer.clear();
            write!(&mut self.buffer, "{}", comb).unwrap();
            ui.centered_and_justified(|ui| {
              let text;

              if let Some(current) = self.current_bind {
                if current == *comb {
                  text = "<press any key>";
                } else {
                  text = &self.buffer;
                }

                if !input.grabbing {
                  self.current_bind = None;

                  let recent = KeyCombination::new(input.state.last_pressed, input.state.modifiers);
                  input.keybinds.rebind(current, recent);
                }
              } else {
                text = &self.buffer;
              }

              let button = egui::Button::new(text);
              let button = button.ui(ui);

              if button.clicked() {
                if self.current_bind.is_none() {
                  self.current_bind = Some(*comb);
                  input.grabbing = true;
                } else {
                  self.current_bind = None;
                  input.grabbing = false;
                }
              }
            });
          });
        });
    }
  }
}
