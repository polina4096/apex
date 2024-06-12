use std::fmt::Write as _;

use egui::Widget as _;

use crate::{
  client::input::client_action::ClientAction,
  core::input::{bind::KeyCombination, Input},
};

use super::GameSettingsView;

impl GameSettingsView {
  pub(super) fn controls_tab(&mut self, ui: &mut egui::Ui, input: &mut Input<ClientAction>) {
    use egui_extras::{Column, TableBuilder};

    let text_height = egui::TextStyle::Body.resolve(ui.style()).size.max(ui.spacing().interact_size.y);
    let available_height = ui.available_height();

    TableBuilder::new(ui)
      .striped(true)
      .resizable(false)
      .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
      .column(Column::auto())
      .column(Column::auto())
      .column(Column::auto())
      .min_scrolled_height(0.0)
      .max_scroll_height(available_height)
      .header(20.0, |mut header| {
        header.col(|ui| {
          ui.strong("Action");
        });
        header.col(|ui| {
          ui.strong("Description");
        });
        header.col(|ui| {
          ui.strong("Keybind");
        });
      })
      .body(|mut body| {
        for (comb, bind) in input.keybinds.as_vec() {
          body.row(text_height, |mut row| {
            row.col(|ui| {
              ui.label(&bind.name);
            });
            row.col(|ui| {
              ui.label(&bind.description);
            });
            row.col(|ui| {
              self.buffer.clear();
              write!(&mut self.buffer, "{}", comb).unwrap();
              ui.centered_and_justified(|ui| {
                let text;

                if let Some(current) = self.current_bind {
                  if current == comb {
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
                    self.current_bind = Some(comb);
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
      });
  }
}
