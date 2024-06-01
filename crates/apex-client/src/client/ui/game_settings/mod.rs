use std::fmt::Write as _;
use egui::Widget;
use log::debug;
use tap::Tap;

use crate::{client::input::client_action::ClientAction, core::input::{bind::{Bind, KeyCombination}, Input}};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameSettingsTab {
  Gameplay,
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
      tab: GameSettingsTab::Gameplay,
      is_open: false,

      buffer: String::new(),
      current_bind: None,
      bind_cache: vec![],
    };
  }

  pub fn prepare(&mut self, ctx: &egui::Context, input: &mut Input<ClientAction>) {
    let mut is_open = self.is_open;

    // TODO: the cache won't be rebuilt if the keybinds are changed while the,
    // settings are open yet it doesn't matter right now as that is not possible.
    if !is_open { self.bind_cache.clear(); return; }
    if self.bind_cache.is_empty() && !input.keybinds.is_empty() {
      debug!("Rebuilding bind cache");
      self.bind_cache = input.keybinds.as_vec();
    }

    ctx.set_visuals(egui::Visuals::dark().tap_mut(|vis| {
      vis.window_highlight_topmost = false;
    }));

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
            let stroke = if self.tab == GameSettingsTab::Gameplay { active } else { default };
            let text = egui::RichText::new("Gameplay").strong().size(16.0);
            let button = egui::Button::new(text).fill(stroke);

            if button.ui(ui).clicked() {
              self.tab = GameSettingsTab::Gameplay;
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
          GameSettingsTab::Gameplay => self.gameplay_tab(ui),
          GameSettingsTab::Controls => self.controls_tab(ui, input),
        }
      });

    self.is_open = is_open;
  }

  fn gameplay_tab(&mut self, ui: &mut egui::Ui) {
    egui::Grid::new("gameplay_tab_grid")
      .num_columns(2)
      .spacing([40.0, 4.0])
      .striped(true)
      .show(ui, |ui| {
        let mut dummy = 0.0;

        {
          ui.label("Audio Offset");
          egui::Slider::new(&mut dummy, -100.0 ..= 100.0)
            .ui(ui);

          ui.end_row();
        }

        {
          ui.label("Zoom");
          egui::Slider::new(&mut dummy, 0.0 ..= 2.0)
            .ui(ui);

          ui.end_row();
        }

        {
          ui.label("Scale");
          egui::Slider::new(&mut dummy, 0.0 ..= 2.0)
            .ui(ui);

          ui.end_row();
        }

        {
          ui.label("Hit position X");
          egui::DragValue::new(&mut dummy)
            .ui(ui);

          ui.end_row();

          ui.label("Hit position Y");
          egui::DragValue::new(&mut dummy)
            .ui(ui);

          ui.end_row();
        }


        {
          use egui::color_picker::{color_edit_button_rgba, Alpha};

          let mut dummy_color = egui::Rgba::from_rgba_unmultiplied(0.8, 0.1, 0.2, 1.0);

          ui.label("Don color");
          color_edit_button_rgba(ui, &mut dummy_color, Alpha::Opaque);
          ui.end_row();

          ui.label("Kat color");
          color_edit_button_rgba(ui, &mut dummy_color, Alpha::Opaque);
          ui.end_row()
        }
      });
  }

  fn controls_tab(&mut self, ui: &mut egui::Ui, input: &mut Input<ClientAction>) {
    use egui_extras::{Column, TableBuilder};

    let text_height = egui::TextStyle::Body
      .resolve(ui.style())
      .size
      .max(ui.spacing().interact_size.y);

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
