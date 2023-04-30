use egui::{Ui, panel::Side};
use wcore::{color::Color, binds::{Bind, Keybinds, KeyCombination, KeyCode}};
use winit::event::{VirtualKeyCode, ModifiersState};

use crate::{view::sidebar::SidebarState, layer::{taiko::TaikoState, Layers}, input::Input};

pub enum AppEvents {
    OpenFilePicker,
    OpenControls,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppKeybinds {
    TogglePlayback,
    ToggleSidebar,

    TimelineMoveForward,
    TimelineMoveBack,
}

pub struct AppState {
    pub keybinds : Keybinds<AppKeybinds>,
    pub input    : Input,

    pub sidebar : SidebarState,
    pub taiko   : TaikoState,
}

impl AppState {
    pub fn new(input: Input) -> Self {
        let mut keybinds = Keybinds::default();

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::Space), modifiers: ModifiersState::empty() },
            Bind { id: AppKeybinds::TogglePlayback, name: String::from("play/pause"), description: String::from("starts or stops playback") }
        );

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::O), modifiers: ModifiersState::CTRL },
            Bind { id: AppKeybinds::ToggleSidebar, name: String::from("toggle sidebar"), description: String::from("shows or hides the sidebar") }
        );

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::Right), modifiers: ModifiersState::empty() },
            Bind { id: AppKeybinds::TimelineMoveForward, name: String::from("Timeline forward"), description: String::from("Move 1/n of a beat forward on a timeline in the song") }
        );

        keybinds.add(
            KeyCombination { key: KeyCode::from(VirtualKeyCode::Left), modifiers: ModifiersState::empty() },
            Bind { id: AppKeybinds::TimelineMoveBack, name: String::from("Timeline back"), description: String::from("Move 1/n of a beat back on a timeline in the song") }
        );

        return Self {
            keybinds,
            input,

            sidebar : SidebarState::new(),
            taiko   : TaikoState::new(),
        };
    }

    pub fn render_settings(&mut self, ui: &mut Ui, layers: &mut Layers) {
        egui::Grid::new("settings")
          .num_columns(2)
          .spacing([40.0, 4.0])
          .striped(true)
          .show(ui, |ui| {
            // Sidebar
            ui.heading("Sidebar");
            ui.end_row();
            
            ui.label("Side");
            egui::ComboBox::from_id_source("side")
              .selected_text(format!("{:?}", self.sidebar.side))
              .show_ui(ui, |ui| {
                ui.style_mut().wrap = Some(false);
                ui.set_min_width(60.0);
                ui.selectable_value(&mut self.sidebar.side, Side::Left, "Left");
                ui.selectable_value(&mut self.sidebar.side, Side::Right, "Right");
            });
            ui.end_row();

            // Taiko
            ui.heading("Taiko");
            ui.end_row();
            
            ui.label("Audio offset");
            ui.add(egui::DragValue::new(&mut self.taiko.audio_offset).suffix("ms"));
            ui.end_row();

            ui.label("Hit position");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.taiko.hit_position.x).suffix("px"));
                ui.add(egui::DragValue::new(&mut self.taiko.hit_position.y).suffix("px"));
            });
            ui.end_row();
            
            ui.label("Scale");
            ui.add(egui::DragValue::new(&mut self.taiko.scale).speed(0.01).min_decimals(2));
            ui.end_row();
            
            ui.label("Zoom");
            if ui.add(egui::DragValue::new(&mut self.taiko.zoom).speed(0.01).min_decimals(2)).changed() {
                layers.taiko.rebuild_pending = true;
            };
            ui.end_row();

            let mut color = [
                (self.taiko.don_color.r * 255.0).round() as u8,
                (self.taiko.don_color.g * 255.0).round() as u8,
                (self.taiko.don_color.b * 255.0).round() as u8,
            ];

            ui.label("Don color");
            if egui::color_picker::color_edit_button_srgb(ui, &mut color).changed() {
                self.taiko.don_color = Color::from_rgb(color[0], color[1], color[2]);
                layers.taiko.rebuild_pending = true;
            };
            ui.end_row();

            color = [
                (self.taiko.kat_color.r * 255.0).round() as u8,
                (self.taiko.kat_color.g * 255.0).round() as u8,
                (self.taiko.kat_color.b * 255.0).round() as u8,
            ];

            ui.label("Kat color");
            if egui::color_picker::color_edit_button_srgb(ui, &mut color).changed() {
                self.taiko.kat_color = Color::from_rgb(color[0], color[1], color[2]);
                layers.taiko.rebuild_pending = true;
            };
            ui.end_row();

            // Debug
            ui.heading("Debug");
            ui.end_row();
            
            ui.label("Force instance rebuild");
            ui.add(egui::Checkbox::without_text(&mut self.taiko.force_rebuild))
                .on_hover_text("Reduces performance by ~25%");
            ui.end_row();
        });
        
    }
}