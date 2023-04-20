use egui::{Align2, vec2, Button, Slider};
use wcore::graphics::{gui::window::Window, context::Graphics};

use crate::{layer::Layers, state::AppState};

const OFFSET: f32 = 12.0;

pub struct TimelineWindow {
    was_playing: bool,
}

impl TimelineWindow {
    pub fn new() -> Self {
        return Self {
            was_playing: false,
        };
    }
}

impl Window<(&mut AppState, &mut Layers)> for TimelineWindow {
    type Title = &'static str;
    fn title() -> Self::Title {
        return "Timeline";
    }

    fn build<'b>(window: egui::Window<'b>, ctx: &'_ egui::Context) -> egui::Window<'b> {
        let rect = ctx.available_rect();
        let size = rect.size();

        window
            .anchor(Align2::CENTER_TOP, vec2(0.0, 96.0))
            .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -OFFSET))
            .fixed_size(vec2(size.x - OFFSET * 3.0, 240.0))
            .collapsible(false)
            .title_bar(false)
    }

    #[allow(unused_variables)]
    fn show(&mut self, (state, layers): (&mut AppState, &mut Layers), view: &wgpu::TextureView, graphics: &mut Graphics, ui: &mut egui::Ui) {
        let time = layers.taiko.get_time().to_ms();
        let length = layers.taiko.get_length();
        
        ui.horizontal(|ui| {
            ui.set_enabled(layers.taiko.beatmap.is_some());

            // Play button
            let play_button_text = if layers.taiko.is_paused() { "▶" } else { "⏸" };
            let play_button = ui.add_sized(vec2(24.0, ui.available_height()), Button::new(play_button_text));
            if play_button.clicked() {
                layers.taiko.toggle_paused();
            };

            // Time display
            ui.label(&format!("{:02}:{:02}:{:03} / {:02}:{:02}:{:03}",
                  time / (60 * 1000),   time / 1000 % 60,   time % 1000,
                length / (60 * 1000), length / 1000 % 60, length % 1000));

            // Time slider
            let slider_width = ui.available_width();
            let style = ui.style_mut();
            style.spacing.slider_width = slider_width;

            let mut time64 = time;
            let slider = Slider::new(&mut time64, 0 ..= (length as u64)).show_value(false);
            let slider = ui.add(slider);               

            if slider.drag_started() {
                self.was_playing = layers.taiko.is_paused();
            }

            if slider.changed() {
                layers.taiko.set_paused(true);
                layers.taiko.set_time(time64 as u32);
            }

            if slider.drag_released() && layers.taiko.beatmap.is_some() {
                layers.taiko.set_paused(self.was_playing);
            }
        }); 
    }
}