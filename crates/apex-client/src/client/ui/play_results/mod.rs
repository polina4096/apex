use std::path::Path;

use egui::{ImageSource, Widget};
use jiff::fmt::strtime;
use kiam::when;
use tap::Tap;

use crate::{
  client::{
    client::Client,
    gameplay::{
      beatmap::{calc_hit_window_150, calc_hit_window_300, Beatmap},
      beatmap_cache::BeatmapInfo,
      taiko_player::TaikoPlayer,
    },
    score::{
      score::Score,
      score_cache::{ScoreCache, ScoreId},
    },
  },
  core::{core::Core, time::time::Time},
};

use super::{
  background_component::BackgroundComponent, beatmap_selection::beatmap_stats::BeatmapStats, ingame_overlay::HitResult,
};

pub struct PlayResultsView {
  background: BackgroundComponent,
  beatmap_stats: BeatmapStats,
  beatmap_info: BeatmapInfo,
  score_id: ScoreId,
  hits: Vec<(Time, i64, HitResult)>,
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

      let mut player = TaikoPlayer::new();
      for hit in score.hits.iter() {
        let hit_window_300 = calc_hit_window_300(beatmap.overall_difficulty);
        let hit_window_150 = calc_hit_window_150(beatmap.overall_difficulty);
        let tolerance = hit_window_150;

        // Check if the hit was within the hit window of the current circle.
        if let Some(circle) = beatmap.hit_objects.get(player.current_circle) {
          let time = circle.time.to_ms();
          let hit_delta = time - hit.to_ms();

          if hit_delta.abs() >= tolerance.to_ms() {
            player.tick(*hit, beatmap.overall_difficulty, &beatmap.hit_objects, |_| {
              hits.push((*hit, hit_delta, HitResult::Miss));
            });
          }

          if hit_delta.abs() < tolerance.to_ms() {
            // if circle.color == TaikoColor::Don
            //   && (input != TaikoPlayerInput::DonOne && input != TaikoPlayerInput::DonTwo)
            // {
            //   return;
            // }

            // if circle.color == TaikoColor::Kat
            //   && (input != TaikoPlayerInput::KatOne && input != TaikoPlayerInput::KatTwo)
            // {
            //   return;
            // }

            player.current_circle += 1;

            when! {
              hit_delta.abs() < hit_window_300.to_ms() => {
                hits.push((*hit, hit_delta, HitResult::Hit300));
              },

              hit_delta.abs() < hit_window_150.to_ms() => {
                hits.push((*hit, hit_delta, HitResult::Hit150));
              },

              _ => {
                hits.push((*hit, hit_delta, HitResult::Miss));
              },
            }
          }
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
    egui::CentralPanel::default().frame(egui::Frame::none()).show(core.egui_ctx(), |ui| {
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

                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                          let width = ui.available_width();
                          let height = ui.available_height();
                          ui.set_width(width);
                          ui.set_height(height);

                          let pos = ui.cursor().min;
                          let mid = pos.y + height / 2.0;

                          let length = self.beatmap_info.length.to_seconds();
                          for (hit, delta, result) in self.hits.iter() {
                            let color = match result {
                              HitResult::Hit300 => egui::Color32::GOLD,
                              HitResult::Hit150 => egui::Color32::LIGHT_BLUE,
                              HitResult::Miss => egui::Color32::RED,
                            };

                            let pos_x = pos.x + (hit.to_seconds() / length * width as f64) as f32;
                            let pos_y = mid + (*delta as f32).clamp(-height / 2.0, height / 2.0);
                            let rect = egui::Rect::from_center_size(egui::pos2(pos_x, pos_y), egui::vec2(2.0, 2.0));
                            ui.painter().rect_filled(rect, 0.0, color);
                          }
                        });
                      });
                    });
                });
            });

            strip.empty();
          });
      });
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
