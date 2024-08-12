use apex_framework::{
  core::Core,
  time::{clock::AbstractClock, time::Time},
};
use delta_bar::{HitDeltaBar, HitDeltaBarOptions};
use instant::Instant;

use crate::client::{
  client::Client,
  gameplay::taiko_player::TaikoInput,
  score::{judgement_processor::Judgement, score_processor::ScoreProcessor},
  settings::Settings,
};

pub mod delta_bar;

pub struct IngameOverlayView {
  last_hit_result_time: Instant,
  last_hit_result_kind: Judgement,

  last_hit_kat_one: Instant,
  last_hit_kat_two: Instant,
  last_hit_don_one: Instant,
  last_hit_don_two: Instant,

  delta_bar: HitDeltaBar,
}

impl IngameOverlayView {
  pub fn new() -> Self {
    return Self {
      last_hit_result_time: Instant::now(),
      last_hit_result_kind: Judgement::Miss,

      last_hit_kat_one: Instant::now(),
      last_hit_kat_two: Instant::now(),
      last_hit_don_one: Instant::now(),
      last_hit_don_two: Instant::now(),

      delta_bar: HitDeltaBar::new(HitDeltaBarOptions {
        bar_width: 128.0,
        bar_height: 24.0,
        bar_opacity: 0.05,
        marker_width: 2.0,
        marker_height: 16.0,
        marker_opacity: 0.25,
        marker_duration: Time::from_seconds(1.0),
        marker_fade: Time::from_seconds(0.2),
      }),
    };
  }

  pub fn update_last_hit_result(&mut self, result: Judgement) {
    if result != Judgement::Hit300 {
      self.last_hit_result_time = Instant::now();
    }

    self.last_hit_result_kind = result;
  }

  pub fn hit(&mut self, delta: Option<Time>, input: TaikoInput) {
    let now = Instant::now();
    if let Some(delta) = delta {
      self.delta_bar.push(delta, now);
    }

    match input {
      TaikoInput::KatOne => {
        self.last_hit_kat_one = now;
      }

      TaikoInput::KatTwo => {
        self.last_hit_kat_two = now;
      }

      TaikoInput::DonOne => {
        self.last_hit_don_one = now;
      }

      TaikoInput::DonTwo => {
        self.last_hit_don_two = now;
      }
    }
  }

  pub fn delta_bar(&mut self) -> &mut HitDeltaBar {
    return &mut self.delta_bar;
  }

  pub fn prepare(
    &mut self,
    core: &mut Core<Client>,
    clock: &mut impl AbstractClock,
    score_processor: &ScoreProcessor,
    hit_window_150: Time,
    hit_window_300: Time,
    settings: &Settings,
  ) {
    egui::CentralPanel::default().frame(egui::Frame::none().inner_margin(egui::Margin::ZERO)).show(
      core.egui.ctx(),
      |ui| {
        let width = ui.available_width();
        let height = ui.available_height();

        self.delta_bar.prepare(ui, width / 2.0, height - 16.0, hit_window_150, hit_window_300);

        let pos = egui::pos2(settings.taiko.hit_position_x(), settings.taiko.hit_position_y());
        let fill = egui::Color32::TRANSPARENT;
        let stroke = egui::Stroke::new(4.0, egui::Color32::GRAY);
        ui.painter().circle(pos, 64.0 * 0.85, fill, stroke);

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

          ui.painter().rect(
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
            Judgement::Hit150 => egui::Color32::from_rgba_unmultiplied( 60, 185, 255, value),
            Judgement::Miss   => egui::Color32::from_rgba_unmultiplied(255,  20,  60, value),

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

        use egui_extras::{Size, StripBuilder};

        StripBuilder::new(ui) //
          .size(Size::remainder())
          .size(Size::exact(128.0))
          .horizontal(|mut strip| {
            strip.cell(|ui| {
              ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                ui.label(egui::RichText::new(format!("(max) {}x", score_processor.max_combo())).size(18.0));
                ui.label(egui::RichText::new(format!("{}x", score_processor.curr_combo())).size(18.0));
              });
            });

            strip.cell(|ui| {
              ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                ui.label(egui::RichText::new(format!("{:.2}%", score_processor.accuracy() * 100.0)).size(24.0));

                {
                  let mut job = egui::text::LayoutJob::default();

                  job.append(
                    &score_processor.result_300s().to_string(),
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
                    &score_processor.result_150s().to_string(),
                    0.0,
                    egui::TextFormat {
                      font_id: egui::FontId::new(18.0, egui::FontFamily::Monospace),
                      color: ui.style().visuals.text_color(),
                      ..Default::default()
                    },
                  );

                  job.append(
                    " 150",
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
                    &score_processor.result_misses().to_string(),
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
            });
          });

        let max_width = ui.available_width();
        let offset = height - 8.0;
        let progress = max_width as f64 / clock.length().to_seconds() * clock.position().to_seconds();
        let rect = egui::Rect::from_min_size(egui::Pos2::new(0.0, offset), egui::Vec2::new(progress as f32, 8.0));
        ui.painter().rect_filled(rect, egui::Rounding::ZERO, egui::Color32::WHITE);

        let font = egui::FontId::proportional(16.0);
        let color = ui.style().visuals.text_color();
        let text = ui.painter().layout_no_wrap(
          format!("{:.2}s", clock.position().to_seconds()),
          font,
          egui::Color32::PLACEHOLDER,
        );
        let pos = egui::pos2(max_width - text.size().x - 4.0, height - text.size().y - 8.0 - 8.0);
        ui.painter().galley(pos, text, color);
      },
    );
  }
}
