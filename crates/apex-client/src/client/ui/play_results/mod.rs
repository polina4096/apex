use std::path::Path;

use apex_framework::{core::Core, time::time::Time};
use egui::ImageSource;
use jiff::fmt::strtime;
use tap::{Pipe, Tap};

use crate::client::{
  client::Client,
  gameplay::{
    beatmap::{calc_hit_window_150, calc_hit_window_300, Beatmap},
    beatmap_cache::BeatmapInfo,
  },
  score::{
    judgement_processor::{check_hit, Judgement},
    score::Score,
    score_cache::{ScoreCache, ScoreId},
  },
};

use super::{background_component::BackgroundComponent, beatmap_selection::beatmap_stats::BeatmapStats};

pub struct PlayResultsView {
  background: BackgroundComponent,
  beatmap_stats: BeatmapStats,
  beatmap_info: BeatmapInfo,
  score_id: ScoreId,
  hits: Vec<(Time, Time, Judgement)>,
}

impl PlayResultsView {
  pub fn new(
    source: impl Into<ImageSource<'static>>,
    beatmap: &Path,
    beatmap_info: BeatmapInfo,
    score_id: ScoreId,
    score_cache: &ScoreCache,
  ) -> Self {
    let image = source.into();
    let background = BackgroundComponent::new(image.clone());
    let beatmap_stats = BeatmapStats::new();

    let score = score_cache.score_details(score_id);
    let mut hits = Vec::with_capacity(score.hits.len());
    if let Ok(data) = std::fs::read_to_string(beatmap) {
      let beatmap = Beatmap::parse(data);

      let mut current_circle = 0;
      for (hit_time, hit_input) in score.hits.iter().copied() {
        let hit_window_300 = calc_hit_window_300(beatmap.overall_difficulty);
        let hit_window_150 = calc_hit_window_150(beatmap.overall_difficulty);

        while let Some(hit_object) = beatmap.hit_objects.get(current_circle) {
          let hit_window_end_time = hit_object.time + hit_window_150;

          if hit_window_end_time >= hit_time {
            if let Some(result) = check_hit(hit_time, hit_object, hit_input, hit_window_150, hit_window_300) {
              hits.push((hit_time, result.hit_delta, result.judgement));

              current_circle += 1;
              break;
            }
          }

          // Unhit hit object which can not be hit anymore counts as a miss.
          hits.push((hit_time, hit_object.time - hit_time, Judgement::Miss));

          current_circle += 1;
        }
      }
    }

    return Self {
      background,
      beatmap_stats,
      beatmap_info,
      score_id,
      hits,
    };
  }

  pub fn prepare(&mut self, core: &Core<Client>, score_cache: &ScoreCache) {
    let score = score_cache.score_details(self.score_id);
    egui::CentralPanel::default().frame(egui::Frame::none()).show(core.egui.ctx(), |ui| {
      self.background.prepare(ui);
      egui::Frame::none().show(ui, |ui| {
        use egui_extras::{Size, StripBuilder};

        StripBuilder::new(ui) //
          .size(Size::remainder())
          .size(Size::relative(0.4))
          .horizontal(|mut strip| {
            strip.cell(|ui| {
              egui::Frame::none() //
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                  self.beatmap_stats.prepare(ui, &self.beatmap_info);

                  ui.add_space(8.0);

                  StripBuilder::new(ui) //
                    .size(Size::exact(113.0))
                    .size(Size::exact(128.0))
                    .vertical(|mut strip| {
                      strip.strip(|builder| {
                        builder //
                          .size(Size::relative(0.4))
                          .size(Size::relative(0.6))
                          .horizontal(|mut strip| {
                            let mut height = 0.0;

                            strip.cell(|ui| {
                              egui::Frame::window(ui.style()) //
                              .outer_margin(egui::Margin::ZERO.tap_mut(|x| x.right = 4.0))
                              .inner_margin(egui::Margin::symmetric(24.0, 16.0))
                              .show(ui, |ui| {
                                height = ui.cursor().min.y;

                                ui.horizontal(|ui| {
                                  ui.add(egui::Label::new(
                                    egui::RichText::new("Score") //
                                      .size(18.0)
                                      .strong(),
                                  ));

                                  ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add(egui::Label::new(
                                      egui::RichText::new(format!("{}", score.score_points())) //
                                        .size(16.0),
                                    ));
                                  });
                                });

                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(8.0);

                                self.render_results_grid(ui, score);
                                height -= ui.cursor().min.y;
                              });
                            });

                            strip.cell(|ui| {
                              self.render_general_info(ui, height.abs(), score);
                            });
                          });
                      });

                      strip.cell(|ui| {
                        ui.add_space(8.0);

                        self.render_hit_delta_graph(ui);
                      });
                    });
                });
            });

            strip.empty();
          });
      });
    });
  }

  fn render_hit_delta_graph(&mut self, ui: &mut egui::Ui) {
    egui::Frame::canvas(ui.style()).rounding(egui::Rounding::same(6.0)).show(ui, |ui| {
      let width = ui.available_width();
      let height = ui.available_height();
      ui.set_width(width);
      ui.set_height(height);

      let pos = ui.cursor().min;
      let mid = pos.y + height / 2.0;

      let hit_window_150 = calc_hit_window_150(self.beatmap_info.overall_difficulty);
      let hit_window_300 = calc_hit_window_300(self.beatmap_info.overall_difficulty);

      let mid_offset_150 = (hit_window_300.to_ms() as f32 / (hit_window_150 * 1.5).to_ms() as f32) * (height / 2.0);
      let mid_offset_miss = (hit_window_150.to_ms() as f32 / (hit_window_150 * 1.5).to_ms() as f32) * (height / 2.0);

      let line_alpha = 8;

      // 300 center line
      let color_300 = egui::Color32::GOLD.pipe(|color| {
        let (r, g, b, _) = color.to_tuple();
        return egui::Color32::from_rgba_unmultiplied(r, g, b, line_alpha);
      });

      ui.painter().line_segment(
        [egui::pos2(pos.x - 1.0, mid), egui::pos2(pos.x + width + 1.0, mid)],
        egui::Stroke::new(1.0, color_300),
      );

      // 150 cut-off lines
      let color_150 = egui::Color32::LIGHT_BLUE.pipe(|color| {
        let (r, g, b, _) = color.to_tuple();
        return egui::Color32::from_rgba_unmultiplied(r, g, b, line_alpha);
      });

      ui.painter().line_segment(
        [
          egui::pos2(pos.x - 1.0, mid + mid_offset_150),
          egui::pos2(pos.x + width + 1.0, mid + mid_offset_150),
        ],
        egui::Stroke::new(1.0, color_150),
      );

      ui.painter().line_segment(
        [
          egui::pos2(pos.x - 1.0, mid - mid_offset_150),
          egui::pos2(pos.x + width + 1.0, mid - mid_offset_150),
        ],
        egui::Stroke::new(1.0, color_150),
      );

      // Miss cut-off lines
      let color_miss = egui::Color32::RED.pipe(|color| {
        let (r, g, b, _) = color.to_tuple();
        return egui::Color32::from_rgba_unmultiplied(r, g, b, line_alpha);
      });

      ui.painter().line_segment(
        [
          egui::pos2(pos.x - 1.0, mid + mid_offset_miss),
          egui::pos2(pos.x + width + 1.0, mid + mid_offset_miss),
        ],
        egui::Stroke::new(1.0, color_miss),
      );

      ui.painter().line_segment(
        [
          egui::pos2(pos.x - 1.0, mid - mid_offset_miss),
          egui::pos2(pos.x + width + 1.0, mid - mid_offset_miss),
        ],
        egui::Stroke::new(1.0, color_miss),
      );

      {
        // Text
        let place = egui::Color32::PLACEHOLDER;

        let font = egui::FontId::proportional(12.0);
        let text = ui.painter().layout_no_wrap(format!("+{}", hit_window_300.to_ms()), font, place);
        let text_pos = egui::pos2(pos.x + 2.0, mid - mid_offset_150 - text.size().y - 1.0);
        ui.painter().galley(text_pos, text, color_150);

        let font = egui::FontId::proportional(12.0);
        let text = ui.painter().layout_no_wrap(format!("+{}", hit_window_150.to_ms()), font, place);
        let text_pos = egui::pos2(pos.x + 2.0, mid - mid_offset_miss - text.size().y - 1.0);
        ui.painter().galley(text_pos, text, color_miss);

        let font = egui::FontId::proportional(12.0);
        let text = ui.painter().layout_no_wrap(format!("-{}", hit_window_300.to_ms()), font, place);
        let text_pos = egui::pos2(pos.x + 2.0, mid + mid_offset_150 + 2.0);
        ui.painter().galley(text_pos, text, color_150);

        let font = egui::FontId::proportional(12.0);
        let text = ui.painter().layout_no_wrap(format!("-{}", hit_window_150.to_ms()), font, place);
        let text_pos = egui::pos2(pos.x + 2.0, mid + mid_offset_miss + 2.0);
        ui.painter().galley(text_pos, text, color_miss);

        let color_text = ui.style().visuals.weak_text_color();
        let font = egui::FontId::proportional(12.0);
        let text = ui.painter().layout_no_wrap(String::from("Late"), font, place);
        let text_pos = egui::pos2(pos.x + width - text.size().x - 4.0, pos.y + 2.0);
        ui.painter().galley(text_pos, text, color_text);

        let font = egui::FontId::proportional(12.0);
        let text = ui.painter().layout_no_wrap(String::from("Early"), font, place);
        let text_pos = egui::pos2(pos.x + width - text.size().x - 4.0, pos.y + height - text.size().y - 2.0);
        ui.painter().galley(text_pos, text, color_text);
      }

      // Hits
      let length = self.beatmap_info.length.to_seconds();
      for (hit, delta, result) in self.hits.iter().copied() {
        let color = match result {
          Judgement::Hit300 => egui::Color32::GOLD,
          Judgement::Hit150 => egui::Color32::LIGHT_BLUE,
          Judgement::Miss => egui::Color32::RED,
        };

        let pos_x = pos.x + (hit.to_seconds() / length * width as f64) as f32;

        let hit_range = hit_window_150 * 1.5; // Make some room for misses.
        let norm_delta = (delta.to_ms() as f32 / hit_range.to_ms() as f32) * (height / 2.0);
        let pos_y = mid + -(norm_delta).clamp(-height / 2.0, height / 2.0);

        let rect = egui::Rect::from_center_size(egui::pos2(pos_x, pos_y), egui::Vec2::splat(2.0));
        ui.painter().rect_filled(rect, 0.0, color);
      }
    });
  }

  fn render_results_grid(&mut self, ui: &mut egui::Ui, score: &Score) {
    egui::Grid::new("results_grid") //
      .num_columns(2)
      .spacing([40.0, 4.0])
      .striped(false)
      .show(ui, |ui| {
        {
          ui.add(egui::Label::new(
            egui::RichText::new("300s") //
              .color(egui::Color32::GOLD)
              .size(16.0)
              .strong(),
          ));

          ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::Label::new(egui::RichText::new(format!("{}x", score.result_300s())).size(16.0).strong()));
          });

          ui.end_row();
        }

        {
          ui.add(egui::Label::new(
            egui::RichText::new("150s") //
              .color(egui::Color32::LIGHT_BLUE)
              .size(16.0)
              .strong(),
          ));

          ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::Label::new(egui::RichText::new(format!("{}x", score.result_150s())).size(16.0).strong()));
          });

          ui.end_row();
        }

        {
          ui.add(egui::Label::new(
            egui::RichText::new("Misses") //
              .color(egui::Color32::DARK_RED)
              .size(16.0)
              .strong(),
          ));

          ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::Label::new(egui::RichText::new(format!("{}x", score.result_misses())).size(16.0).strong()));
          });

          ui.end_row();
        }
      });
  }

  fn render_general_info(&self, ui: &mut egui::Ui, height: f32, score: &Score) {
    egui::Frame::window(ui.style()) //
      .outer_margin(egui::Margin::ZERO.tap_mut(|x| x.right = 8.0))
      .inner_margin(egui::Margin::symmetric(24.0, 16.0))
      .show(ui, |ui| {
        use egui_extras::{Size, StripBuilder};

        StripBuilder::new(ui) //
          .size(Size::exact(height - 18.0 - 6.0 - 17.0))
          .size(Size::exact(16.0))
          .vertical(|mut strip| {
            strip.cell(|ui| {
              ui.horizontal(|ui| {
                ui.vertical(|ui| {
                  ui.add(egui::Label::new(
                    egui::RichText::new("Max Combo") //
                      .size(16.0),
                  ));

                  ui.add(egui::Label::new(egui::RichText::new(format!("{}x", score.max_combo())).size(21.0).strong()));
                });

                ui.add_space(8.0);

                ui.vertical(|ui| {
                  ui.add(egui::Label::new(
                    egui::RichText::new("Accuracy") //
                      .size(16.0),
                  ));

                  ui.add(egui::Label::new(
                    egui::RichText::new(format!("{:.2}%", score.accuracy() * 100.0)).size(21.0).strong(),
                  ));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                  let grade = score.grade();

                  ui.add_space(4.0);

                  ui.add(egui::Label::new(
                    egui::RichText::new(grade.to_string()) //
                      .color(grade.color())
                      .size(28.0)
                      .strong(),
                  ));
                });
              });
            });

            strip.cell(|ui| {
              ui.separator();
              ui.add_space(8.0);
              ui.horizontal(|ui| {
                ui.label(format!("Played by {}", score.username()));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                  let str = strtime::format("%H:%M:%S@%Y-%m-%d", score.date()).unwrap();
                  let (a, b) = str.split_once("@").unwrap();
                  ui.label(format!("{}  ∙  {}", a, b));
                });
              });
            });
          });
      });
  }
}
