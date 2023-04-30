use std::{sync::mpsc, slice, hash::Hash, fmt::Write};

use egui::Button;
use egui_extras::{TableBuilder, Column, TableBody};
use log::error;
use wcore::{graphics::{gui::window::Window, context::Graphics}, binds::{KeyCombination, Bind}};

use crate::{layer::Layers, state::AppState, input::Input};

#[derive(Clone, Copy, PartialEq)]
enum ControlGroup {
    None,
    General,
    Taiko,
}

pub struct ControlsWindow {
    open : bool,

    input_receiver : mpsc::Receiver<()>,
    control_group  : ControlGroup,
    old_key        : KeyCombination,
}

impl ControlsWindow {
    pub fn new(input_receiver: mpsc::Receiver<()>) -> Self {
        return Self {
            open : false,

            input_receiver : input_receiver,
            control_group  : ControlGroup::None,
            old_key        : Default::default()
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
                .striped(false)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::remainder())
                .column(Column::initial(100.0))
                .min_scrolled_height(0.0);

            let mut key_text = String::new();
            table.body(|mut body| {
                fn show_keybinds<T: Copy + Eq + Hash>(
                    body     : &mut TableBody,
                    iter     : slice::Iter<'_, (KeyCombination, Bind<T>)>,
                    input    : &mut Input,
                    old_key  : &mut KeyCombination,
                    key_text : &mut String,
                    n_group  : &mut ControlGroup,
                    c_group  : ControlGroup,
                ) {
                    for (key, bind) in iter {
                        body.row(18.0, |mut row| {
                            row.col(|ui| { ui.strong(&bind.name); });
                            row.col(|ui| { ui.label(&bind.description); });
                            row.col(|ui| {
                                if input.requests_input && old_key == key && *n_group == c_group {
                                    key_text.clear();
                                    key_text.push_str("<press any key>");
                                } else {
                                    key_text.clear();
                                    write!(key_text, "{}", key)
                                        .unwrap_or_else(|err| error!("{}", err));
                                }
    
                                let keybind_button = ui.add(Button::new(&*key_text)
                                    .min_size(egui::Vec2::new(100.0, 18.0)));
    
                                if keybind_button.clicked() {
                                    input.requests_input = !input.requests_input;
                                    *old_key = *key;
                                    *n_group = c_group;
                                }
                            });
                        });
                    }
                }

                body.row(18.0, |mut row| { row.col(|ui| { ui.heading("General"); }); });
                show_keybinds(&mut body, state.keybinds.iter(), &mut state.input, &mut self.old_key, &mut key_text, &mut self.control_group, ControlGroup::General);

                body.row(18.0, |mut row| { row.col(|ui| { ui.heading("Taiko"); }); });
                show_keybinds(&mut body, state.taiko.keybinds.iter(), &mut state.input, &mut self.old_key, &mut key_text, &mut self.control_group, ControlGroup::Taiko);
            });

            if self.input_receiver.try_recv().is_ok() {
                let new_key = KeyCombination {
                    key       : state.input.key,
                    modifiers : state.input.modifiers,
                };

                match self.control_group {
                    ControlGroup::General => state.keybinds       . rebind(self.old_key, new_key),
                    ControlGroup::Taiko   => state.taiko.keybinds . rebind(self.old_key, new_key),

                    ControlGroup::None => { },
                }
            }
        });
    }

    fn set_visible(&mut self, value: bool) { self.open = value; }
    fn get_visible(&self) -> bool { self.open }
}