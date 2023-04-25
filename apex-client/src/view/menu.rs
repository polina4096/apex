use egui::{TopBottomPanel, menu};
use wcore::{graphics::{gui::{view::View, window::Window}, context::Graphics}};

use crate::{state::AppState, layer::Layers};

use super::window::file_dialog::FileDialogWindow;

pub struct MenuView {}

impl MenuView {
    pub fn new() -> Self {
        return Self {};
    }
}

impl View<(&mut AppState, &mut Layers, &mut FileDialogWindow)> for MenuView {
    #[allow(unused_variables)]
    fn show(&mut self, (state, layers, file_dialog): (&mut AppState, &mut Layers, &mut FileDialogWindow), view: &wgpu::TextureView, graphics: &mut Graphics, ctx: &egui::Context) {
        TopBottomPanel::top("menu").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        file_dialog.set_visible(true);
                        ui.close_menu();
                    }

                    if ui.button("Close").clicked() {
                        layers.taiko.unload_beatmap();
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("View", |ui| {
                    if ui.button(format!("{} Hit circles", if state.taiko.hit_circles { "✔" } else { "❌" })).clicked() {
                        state.taiko.hit_circles = !state.taiko.hit_circles;
                        ui.close_menu();
                    }
                });

                ui.with_layout(egui::Layout::default().with_cross_align(egui::Align::Max), |ui| {
                    let value = ui.available_height();
                    if ui.add_sized(egui::vec2(value, value), egui::Button::new("⛭")).clicked() {
                        state.sidebar.shown = !state.sidebar.shown;
                    }
                });
            });
        });
    }
}