use instant::Instant;

use crate::{
  client::{
    client::Client,
    gameplay::{score_processor::ScoreProcessor, taiko_player::TaikoPlayerInput},
    settings::Settings,
  },
  core::{core::Core, time::clock::AbstractClock},
};

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

  pub fn prepare(
    &mut self,
    core: &mut Core<Client>,
    clock: &mut impl AbstractClock,
    score: &ScoreProcessor,
    settings: &Settings,
  ) {
    egui::CentralPanel::default().frame(egui::Frame::none().inner_margin(egui::Margin::same(8.0))).show(
      core.egui_ctx(),
      |ui| {
        let max_height = ui.available_height();

        let painter = ui.painter();

        let pos = egui::pos2(settings.taiko.hit_position_x(), settings.taiko.hit_position_y());
        let fill = egui::Color32::TRANSPARENT;
        let stroke = egui::Stroke::new(4.0, egui::Color32::GRAY);
        painter.circle(pos, 64.0 * 0.85, fill, stroke);

        let draw_hit_key = |i: f32, elapsed: f32| {
          let fade = 0.2;
          let max_brightness = 200;
          let base_brightness = 40;
          let value = elapsed.min(fade) / fade * max_brightness as f32;

          let size = egui::vec2(32.0, 32.0);
          let pos = egui::pos2(
            settings.taiko.hit_position_x() + i * (4.0 + size.x) - 2.0 * (4.0 + size.x),
            settings.taiko.hit_position_y() + 70.0,
          );

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

          #[rustfmt::skip]
          let color = match self.last_hit_result_kind {
            HitResult::Hit100 => egui::Color32::from_rgba_unmultiplied( 60, 185, 255, value),
            HitResult::Miss   => egui::Color32::from_rgba_unmultiplied(255,  20,  60, value),

            _ => { break 'a }
          };

          let value = elapsed.min(0.125) * 1.25;
          let value = 1.05 + value;

          painter.circle(
            egui::pos2(settings.taiko.hit_position_x(), settings.taiko.hit_position_y()),
            64.0 * 0.55 * value,
            color,
            egui::Stroke::NONE,
          );
        }

        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
          ui.label(egui::RichText::new(format!("{:.2}%", score.accuracy() * 100.0)).size(24.0));

          {
            let mut job = egui::text::LayoutJob::default();

            job.append(
              &score.result_300s().to_string(),
              0.0,
              egui::TextFormat {
                font_id: egui::FontId::new(18.0, egui::FontFamily::Monospace),
                color: ui.style().visuals.text_color(),
                ..Default::default()
              },
            );

            job.append(
              " 300",
              0.0,
              egui::TextFormat {
                font_id: egui::FontId::new(18.0, egui::FontFamily::Monospace),
                color: egui::Color32::GOLD,
                ..Default::default()
              },
            );

            ui.label(job);
          }

          {
            let mut job = egui::text::LayoutJob::default();

            job.append(
              &score.result_100s().to_string(),
              0.0,
              egui::TextFormat {
                font_id: egui::FontId::new(18.0, egui::FontFamily::Monospace),
                color: ui.style().visuals.text_color(),
                ..Default::default()
              },
            );

            job.append(
              " 100",
              0.0,
              egui::TextFormat {
                font_id: egui::FontId::new(18.0, egui::FontFamily::Monospace),
                color: egui::Color32::LIGHT_BLUE,
                ..Default::default()
              },
            );

            ui.label(job);
          }

          {
            let mut job = egui::text::LayoutJob::default();

            job.append(
              &score.result_misses().to_string(),
              0.0,
              egui::TextFormat {
                font_id: egui::FontId::new(18.0, egui::FontFamily::Monospace),
                color: ui.style().visuals.text_color(),
                ..Default::default()
              },
            );

            job.append(
              " bad",
              0.0,
              egui::TextFormat {
                font_id: egui::FontId::new(18.0, egui::FontFamily::Monospace),
                color: egui::Color32::DARK_RED,
                ..Default::default()
              },
            );

            ui.label(job);
          }
        });

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Max), |ui| {
          let max_width = ui.available_width();
          let offset = max_height + 8.0;
          let progress = max_width as f64 / clock.length().to_seconds() * clock.position().to_seconds();
          let rect = egui::Rect::from_min_size(egui::Pos2::new(0.0, offset), egui::Vec2::new(progress as f32, 8.0));
          ui.painter().rect_filled(rect, egui::Rounding::ZERO, egui::Color32::WHITE);

          ui.label(egui::RichText::new(format!("{}", clock.position())).size(16.0));
        });
      },
    );
  }
}
