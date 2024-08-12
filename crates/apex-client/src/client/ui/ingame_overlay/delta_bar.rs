use std::collections::VecDeque;

use apex_framework::time::time::Time;
use instant::Instant;

pub struct HitDeltaBar {
  hit_deltas: VecDeque<(Time, Instant)>,

  bar_width: f32,
  bar_height: f32,
  bar_opacity: f32,

  marker_width: f32,
  marker_height: f32,
  marker_opacity: f32,
  marker_duration: Time,
  marker_fade: Time,
}

pub struct HitDeltaBarOptions {
  pub bar_width: f32,
  pub bar_height: f32,
  pub bar_opacity: f32,
  pub marker_width: f32,
  pub marker_height: f32,
  pub marker_opacity: f32,
  pub marker_duration: Time,
  pub marker_fade: Time,
}

impl HitDeltaBar {
  pub fn new(options: HitDeltaBarOptions) -> Self {
    return Self {
      hit_deltas: VecDeque::new(),
      bar_width: options.bar_width,
      bar_height: options.bar_height,
      bar_opacity: options.bar_opacity,
      marker_width: options.marker_width,
      marker_height: options.marker_height,
      marker_opacity: options.marker_opacity,
      marker_duration: options.marker_duration,
      marker_fade: options.marker_fade,
    };
  }

  pub fn prepare(
    &mut self,
    ui: &mut egui::Ui,
    x_offset: f32,
    y_offset: f32,
    hit_window_150: Time,
    hit_window_300: Time,
  ) {
    let center_300_offset_x = hit_window_300.to_ms() as f32 / hit_window_150.to_ms() as f32 * self.bar_width;
    let a = (255.0 * self.bar_opacity).round() as u8;

    // 300 color range
    {
      let color = {
        let (r, g, b, _) = egui::Color32::GOLD.to_tuple();
        egui::Color32::from_rgba_unmultiplied(r, g, b, a)
      };

      let rect = egui::Rect::from_two_pos(
        egui::pos2(x_offset - center_300_offset_x, y_offset - self.marker_height),
        egui::pos2(x_offset + center_300_offset_x, y_offset),
      );
      ui.painter().rect_filled(rect, egui::Rounding::ZERO, color);
    }

    // 150 color ranges
    {
      let color = {
        let (r, g, b, _) = egui::Color32::LIGHT_BLUE.to_tuple();
        egui::Color32::from_rgba_unmultiplied(r, g, b, a)
      };

      let rect = egui::Rect::from_two_pos(
        egui::pos2(x_offset - self.bar_width, y_offset - self.marker_height),
        egui::pos2(x_offset - center_300_offset_x, y_offset),
      );
      ui.painter().rect_filled(rect, egui::Rounding::ZERO, color);

      let rect = egui::Rect::from_two_pos(
        egui::pos2(x_offset + center_300_offset_x, y_offset - self.marker_height),
        egui::pos2(x_offset + self.bar_width, y_offset),
      );
      ui.painter().rect_filled(rect, egui::Rounding::ZERO, color);
    }

    // Middle marker
    let pos = egui::pos2(x_offset, y_offset + self.marker_height / 2.0 - self.marker_height);
    let rect = egui::Rect::from_center_size(pos, egui::vec2(self.marker_width, self.bar_height));
    ui.painter().rect_filled(rect, egui::Rounding::ZERO, egui::Color32::LIGHT_GRAY);

    // Hit deltas
    self.hit_deltas.retain(|(delta, instant)| {
      let elapsed = instant.elapsed();
      if elapsed.as_secs_f64() > self.marker_duration.to_seconds() {
        return false;
      }

      let max_alpha = 255.0 * self.marker_opacity;
      let fade = (self.marker_duration.to_seconds() / self.marker_fade.to_seconds()) as f32;
      let value = ((1.0 - (elapsed.as_secs_f32() - 0.5).abs() * 2.0) * fade).min(1.0) * max_alpha;
      let a = value.round() as u8;

      let color = if delta.abs() <= hit_window_300 {
        let (r, g, b, _) = egui::Color32::GOLD.to_tuple();
        egui::Color32::from_rgba_unmultiplied(r, g, b, a)
      } else {
        let (r, g, b, _) = egui::Color32::LIGHT_BLUE.to_tuple();
        egui::Color32::from_rgba_unmultiplied(r, g, b, a)
      };

      let value = (delta.to_ms() as f32 / hit_window_150.to_ms() as f32 * self.bar_width)
        .clamp(-self.bar_width / 2.0, self.bar_width / 2.0);

      let pos = egui::pos2(x_offset + value, y_offset + self.marker_height / 2.0 - self.marker_height);
      let rect = egui::Rect::from_center_size(pos, egui::vec2(self.marker_width, self.marker_height));
      ui.painter().rect_filled(rect, egui::Rounding::ZERO, color);

      return true;
    });
  }

  pub fn push(&mut self, delta: Time, instant: Instant) {
    self.hit_deltas.push_back((delta, instant));
  }

  pub fn set_bar_width(&mut self, value: f32) {
    self.bar_width = value;
  }

  pub fn set_bar_height(&mut self, value: f32) {
    self.bar_height = value;
  }

  pub fn set_bar_opacity(&mut self, value: f32) {
    self.bar_opacity = value;
  }

  pub fn set_marker_width(&mut self, value: f32) {
    self.marker_width = value;
  }

  pub fn set_marker_height(&mut self, value: f32) {
    self.marker_height = value;
  }

  pub fn set_marker_opacity(&mut self, value: f32) {
    self.marker_opacity = value;
  }

  pub fn set_marker_duration(&mut self, value: Time) {
    self.marker_duration = value;
  }

  pub fn set_marker_fade(&mut self, value: Time) {
    self.marker_fade = value;
  }
}
