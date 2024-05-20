use instant::Instant;

use crate::{client::{client::Client, gameplay::taiko_player::TaikoPlayerInput, screen::gameplay_screen::playback_controller::PlaybackController}, core::{core::Core, time::time::Time}};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HitResult {
  Hit300,
  Hit100,
  Miss,
}

pub struct IngameOverlayView {
  last_hit_result_time: Instant,
  last_hit_result_kind: HitResult,

  last_hit_kat_one: Instant,
  last_hit_kat_two: Instant,
  last_hit_don_one: Instant,
  last_hit_don_two: Instant,
}

impl IngameOverlayView {
  pub fn new() -> Self {
    return Self {
      last_hit_result_time: Instant::now(),
      last_hit_result_kind: HitResult::Miss,

      last_hit_kat_one: Instant::now(),
      last_hit_kat_two: Instant::now(),
      last_hit_don_one: Instant::now(),
      last_hit_don_two: Instant::now(),
    };
  }

  pub fn show_hit_result(&mut self, result: HitResult) {
    if result != HitResult::Hit300 {
      self.last_hit_result_time = Instant::now();
    }

    self.last_hit_result_kind = result;
  }

  pub fn hit(&mut self, input: TaikoPlayerInput) {
    match input {
      TaikoPlayerInput::KatOne => {
        self.last_hit_kat_one = Instant::now();
      }

      TaikoPlayerInput::KatTwo => {
        self.last_hit_kat_two = Instant::now();
      }

      TaikoPlayerInput::DonOne => {
        self.last_hit_don_one = Instant::now();
      }

      TaikoPlayerInput::DonTwo => {
        self.last_hit_don_two = Instant::now();
      }
    }
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, mut playback: impl PlaybackController) {
    egui::CentralPanel::default()
      .frame(egui::Frame::none().inner_margin(egui::Margin::same(8.0)))
      .show(core.egui_ctx.egui_ctx(), |ui| {
        let painter = ui.painter();
        painter.circle(egui::pos2(150.0, 150.0), 64.0 * 0.85, egui::Color32::TRANSPARENT, egui::Stroke::new(4.0, egui::Color32::GRAY));

        let draw_hit_key = |i: f32, elapsed: f32| {
          let fade = 0.2;
          let max_brightness = 200;
          let base_brightness = 40;
          let value = elapsed.min(fade) / fade * max_brightness as f32;

          let size = egui::vec2(32.0, 32.0);
          let pos = egui::pos2(150.0 + i * (4.0 + size.x) - 2.0 * (4.0 + size.x), 220.0);

          painter.rect(
            egui::Rect::from_min_size(pos, size),
            egui::Rounding::ZERO,
            egui::Color32::from_gray(max_brightness + base_brightness - value.round() as u8),
            egui::Stroke::new(2.0, egui::Color32::WHITE),
          );
        };

        draw_hit_key(0.0, self.last_hit_kat_one.elapsed().as_secs_f32());
        draw_hit_key(1.0, self.last_hit_don_one.elapsed().as_secs_f32());
        draw_hit_key(2.0, self.last_hit_don_two.elapsed().as_secs_f32());
        draw_hit_key(3.0, self.last_hit_kat_two.elapsed().as_secs_f32());

        'a: {
          let painter = ui.painter();

          let elapsed = self.last_hit_result_time.elapsed().as_secs_f32();

          let fade = 0.4;
          let max_brightness = 150;
          let base_brightness = 0;
          let value = elapsed.min(fade) / fade * max_brightness as f32;
          let value = max_brightness + base_brightness - value.round() as u8;

          let color = match self.last_hit_result_kind {
            HitResult::Hit100 => egui::Color32::from_rgba_unmultiplied( 60, 185, 255, value),
            HitResult::Miss   => egui::Color32::from_rgba_unmultiplied(255,  20,  60, value),

            _ => { break 'a }
          };

          let value = elapsed.min(0.125) * 1.25;
          let value = 1.05 + value;

          painter.circle(egui::pos2(150.0, 150.0), 64.0 * 0.55 * value, color, egui::Stroke::NONE);
        }

        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
          ui.label(egui::RichText::new("accuracy or whatever").size(16.0));
        });

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Max), |ui| {
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
