use egui::{TopBottomPanel, menu};
use log::error;
use wcore::{graphics::{gui::view::View, context::Graphics}};
use winit::event_loop::EventLoopProxy;

use crate::{state::{AppState, AppEvents}, layer::Layers};

pub struct MenuView {}

impl MenuView {
    pub fn new() -> Self {
        return Self {};
    }
}

impl View<(&mut AppState, &mut Layers, &EventLoopProxy<AppEvents>)> for MenuView {
    #[allow(unused_variables)]
    fn show(&mut self, (state, layers, proxy): (&mut AppState, &mut Layers, &EventLoopProxy<AppEvents>), view: &wgpu::TextureView, graphics: &mut Graphics, ctx: &egui::Context) {
        TopBottomPanel::top("menu").show(ctx, |ui| {
            menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        proxy.send_event(AppEvents::OpenFilePicker)
                            .unwrap_or_else(|err| error!("{}", err));
                        
                        ui.close_menu();
                    }

                    if ui.button("Close").clicked() {
                        layers.taiko.unload_beatmap();
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Edit", |ui| {
                    if ui.button("Controls").clicked() {
                        proxy.send_event(AppEvents::OpenControls)
                            .unwrap_or_else(|err| error!("{}", err));

                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button(format!("{} Hit circles", if state.taiko.hide_circles { "✔" } else { "❌" })).clicked() {
                        state.taiko.hide_circles = !state.taiko.hide_circles;
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