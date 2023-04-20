use egui::panel::Side;
use wcore::graphics::{gui::view::View, context::Graphics};

use crate::{state::AppState, layer::Layers};

pub struct SidebarState {
    pub shown : bool,
    pub side  : Side,
}

impl SidebarState {
    pub fn new() -> Self {
        return Self {
            shown : false,
            side  : Side::Right,
        };
    }
}

pub struct SidebarView { }

impl SidebarView {
    pub fn new() -> Self {
        return Self { };
    }
}

impl View<(&mut AppState, &mut Layers)> for SidebarView {
    #[allow(unused_variables)]
    fn show(&mut self, (state, layers): (&mut AppState, &mut Layers), view: &wgpu::TextureView, graphics: &mut Graphics, ctx: &egui::Context) {
        egui::SidePanel::new(state.sidebar.side, "sidebar")
          .show_animated(ctx, state.sidebar.shown, |ui| {
            egui::ScrollArea::new([false, true]).show(ui, |ui| {
                state.render_settings(ui);
            });
        });
    }
}