use crate::{client::{client::Client, screen::gameplay_screen::playback_controller::PlaybackController}, core::{core::Core, time::time::Time}};

pub struct IngameOverlayView {
}

impl IngameOverlayView {
  pub fn new() -> Self {
    return Self { };
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, mut playback: impl PlaybackController) {
    egui::CentralPanel::default()
      .frame(egui::Frame::none().inner_margin(egui::Margin::same(8.0)))
      .show(core.egui_ctx.egui_ctx(), |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
          ui.label(egui::RichText::new("accuracy or whatever").size(16.0));
        });

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Max), |ui| {
          // let max_width = ui.available_width();
          // let offset = max_height + 8.0;
          // let progress = max_width as f64 / clock.get_length().to_seconds() * clock.get_time().to_seconds();
          // let rect = egui::Rect::from_min_size(egui::Pos2::new(0.0, offset), egui::Vec2::new(progress as f32, 8.0));
          // ui.painter().rect_filled(rect, egui::Rounding::ZERO, egui::Color32::WHITE);

          let max_width = ui.available_width();
          let song_length = playback.length().to_seconds();
          let mut song_position = playback.position().to_seconds();
          ui.spacing_mut().slider_width = max_width;
          let slider = ui.add(egui::Slider::new(&mut song_position, 0.0 ..= song_length).show_value(false));

          // TODO: causes desync without clock synchronization
          // if slider.drag_started() {
          //   playback.set_playing(false);
          // }
          //
          // if slider.drag_stopped() {
          //   playback.set_playing(true);
          // }

          if slider.changed() {
            playback.set_position(Time::from_seconds(song_position));
          };

          ui.label(egui::RichText::new(format!("{}", playback.position())).size(16.0));
        });
      });
  }
}
