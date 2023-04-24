use egui_file::FileDialog;
use wcore::{graphics::{gui::{view::View, window::Window}, context::Graphics}};

use crate::{state::AppState, layer::Layers};

pub struct FileDialogWindow {
    open   : bool,
    dialog : FileDialog,
}

impl FileDialogWindow {
    pub fn new() -> Self {
        return Self {
            open   : false,
            dialog : FileDialog::open_file(None),
        };
    }
}

impl Window<()> for FileDialogWindow {
    type Title = &'static str;
    fn title() -> Self::Title {
        return "Choose a file";
    }

    fn build<'b>(window: egui::Window<'b>, _ctx: &'_ egui::Context) -> egui::Window<'b> {
        window
            .collapsible(false)
            .resizable(true)
            .default_pos(egui::pos2(8.0, 32.0))
            .title_bar(true)
    }

    fn set_visible(&mut self, value: bool) { self.open = value; if value { self.dialog.open(); } }
    fn get_visible(&self) -> bool { return self.open; }

    #[allow(unused_variables)]
    fn show(&mut self, state: (), view: &wgpu::TextureView, graphics: &mut Graphics, ui: &mut egui::Ui) { }
}

// Hand-rolling a window view impl 
impl View<(&mut AppState, &mut Layers)> for FileDialogWindow {
    #[allow(unused_variables)]
    fn show(&mut self, (state, layers): (&mut AppState, &mut Layers), view: &wgpu::TextureView, graphics: &mut Graphics, ctx: &egui::Context) {
        if self.dialog.show(ctx).selected() {
            if let Some(path) = self.dialog.path() {
                layers.taiko.open_beatmap(path.as_path(), &mut state.taiko);
            }
        }
    }
}