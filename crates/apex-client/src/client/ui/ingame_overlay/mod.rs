use apex_framework::{
  core::Core,
  time::{clock::AbstractClock, time::Time},
};
use delta_bar::{HitDeltaBar, HitDeltaBarOptions};
use instant::Instant;

use crate::client::{client::Client, score::score_processor::ScoreProcessor};

pub mod delta_bar;

pub struct IngameOverlayView {
  delta_bar: HitDeltaBar,
}

impl IngameOverlayView {
  pub fn new() -> Self {
    return Self {
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

  pub fn hit(&mut self, delta: Time) {
    let now = Instant::now();
    self.delta_bar.push(delta, now);
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
  ) {
    egui::CentralPanel::default().frame(egui::Frame::none().inner_margin(egui::Margin::ZERO)).show(
      core.egui.ctx(),
      |ui| {
        let width = ui.available_width();
        let height = ui.available_height();

        self.delta_bar.prepare(ui, width / 2.0, height - 16.0, hit_window_150, hit_window_300);

        // let draw_hit_key = |i: f32, elapsed: f32| {
        //   let fade = 0.2;
        //   let max_brightness = 200;
        //   let base_brightness = 40;
        //   let value = elapsed.min(fade) / fade * max_brightness as f32;

        //   let size = egui::vec2(32.0, 32.0);
        //   let pos = egui::pos2(
        //     settings.taiko.hit_position_x_px() + i * (4.0 + size.x) - 2.0 * (4.0 + size.x),
        //     core.graphics.config.height as f32 * settings.taiko.hit_position_y_perc() + 70.0,
        //   );

        //   ui.painter().rect(
        //     egui::Rect::from_min_size(pos, size),
        //     egui::Rounding::ZERO,
        //     egui::Color32::from_gray(max_brightness + base_brightness - value.round() as u8),
        //     egui::Stroke::new(2.0, egui::Color32::WHITE),
        //   );
        // };

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
