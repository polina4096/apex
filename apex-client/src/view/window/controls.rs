use std::{sync::mpsc, fmt::Write};

use egui::Button;
use egui_extras::{TableBuilder, Column};
use log::error;
use wcore::{graphics::{gui::window::Window, context::Graphics}, binds::KeyCombination};

use crate::{layer::Layers, state::AppState};

pub struct ControlsWindow {
    open : bool,

    input_receiver : mpsc::Receiver<()>,
    old_key     : KeyCombination,
}

impl ControlsWindow {
    pub fn new(input_receiver: mpsc::Receiver<()>) -> Self {
        return Self {
            open  : false,
            input_receiver : input_receiver,
            old_key     : Default::default()
        };
    }
}

impl Window<(&mut AppState, &mut Layers)> for ControlsWindow {
    type Title = &'static str;
    fn title() -> Self::Title {
        return "Controls";
    }

    #[allow(unused_variables)]
    fn build<'b>(window: egui::Window<'b>, ctx: &'_ egui::Context) -> egui::Window<'b> {
        window
            .default_height(256.0 + 128.0)
            .default_pos((16.0, 16.0 + 24.0))
    }

    #[allow(unused_variables)]
    fn show(&mut self, (state, layers): (&mut AppState, &mut Layers), view: &wgpu::TextureView, graphics: &mut Graphics, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
          .auto_shrink([true, true])
          .show(ui, |ui| {
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::remainder())
                .column(Column::initial(100.0))
                .min_scrolled_height(0.0);

            let mut key_text = String::new();
            table.body(|mut body| {
                for (key, bind) in state.keybinds.iter() {
                    body.row(18.0, |mut row| {
                        row.col(|ui| { ui.strong(&bind.name); });
                        row.col(|ui| { ui.label(&bind.description); });
                        row.col(|ui| {

                            if state.input.requests_input && self.old_key == *key {
                                key_text.clear();
                                key_text.push_str("<press any key>");
                            } else {
                                key_text.clear();
                                write!(&mut key_text, "{}", key)
                                    .unwrap_or_else(|err| error!("{}", err));
                            }

                            let keybind_button = ui.add(Button::new(&key_text)
                                .min_size(egui::Vec2::new(100.0, 18.0)));

                            if keybind_button.clicked() {
                                state.input.requests_input = !state.input.requests_input;
                                self.old_key = *key;
                            }
                        });
                    });
                }
            });

            if self.input_receiver.try_recv().is_ok() {
                let new_key = KeyCombination {
                    key       : state.input.key,
                    modifiers : state.input.modifiers,
                };

                state.keybinds.rebind(self.old_key, new_key);
            }
        });
    }

    fn set_visible(&mut self, value: bool) { self.open = value; }
    fn get_visible(&self) -> bool { self.open }
}