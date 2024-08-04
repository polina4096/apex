use egui::{ImageSource, Widget as _};
use jiff::fmt::strtime;
use tap::Tap;

use crate::{
  client::{
    client::Client,
    gameplay::beatmap_cache::BeatmapInfo,
    score::{
      score::Score,
      score_cache::{ScoreCache, ScoreId},
    },
  },
  core::core::Core,
};

use super::{
  background_component::BackgroundComponent, beatmap_selection::beatmap_stats::BeatmapStats,
  card_component::CardComponent,
};

pub struct PlayResultsView {
  background: BackgroundComponent,
  beatmap_stats: BeatmapStats,
  beatmap_info: BeatmapInfo,
  score_id: ScoreId,
}

impl PlayResultsView {
  pub fn new(source: impl Into<ImageSource<'static>>, beatmap_info: BeatmapInfo, score_id: ScoreId) -> Self {
    let image = source.into();
    let background = BackgroundComponent::new(image.clone());
    let beatmap_stats = BeatmapStats::new();

    return Self {
      background,
      beatmap_stats,
      beatmap_info,
      score_id,
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
            });

            strip.cell(|ui| {});
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
            ui.add(egui::Label::new(egui::RichText::new(format!("{}x", score.result_150s())).size(16.0).strong()));
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
          .size(Size::exact(height - 18.0 - 6.0 - 18.0))
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
