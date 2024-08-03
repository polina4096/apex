use std::fmt::Write as _;

use egui::Widget;
use jiff::Timestamp;
use tap::Tap as _;

use crate::client::score::{
  score::Score,
  score_cache::{ScoreCache, ScoreId},
};

pub struct BeatmapScores {
  buffer: String,
}

impl BeatmapScores {
  pub fn new() -> Self {
    return Self { buffer: String::new() };
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui, score_chache: &ScoreCache, score_ids: &[ScoreId]) {
    egui::Frame::window(ui.style())
      .outer_margin(egui::Margin {
        left: 12.0,
        right: 12.0,
        top: 6.0,
        bottom: 0.0,
      })
      .inner_margin(egui::Margin::symmetric(12.0, 8.0))
      .show(ui, |ui| {
        ui.set_width(ui.available_width().min(512.0 + 32.0));
        ui.horizontal(|ui| {
          write!(&mut self.buffer, "Scores ({})", score_ids.len()).unwrap();
          ui.heading(&self.buffer);
          self.buffer.clear();

          ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(egui::RichText::new("Global").weak());
            ui.weak("∙");
            ui.label(egui::RichText::new("Local"));
            ui.weak("∙");
            ui.label(egui::RichText::new("Details").weak());
          });
        });
      });

    egui::Frame::none()
      .outer_margin(egui::Margin {
        left: 0.0,
        right: 12.0,
        top: 0.0,
        bottom: 0.0,
      })
      .inner_margin(egui::Margin::symmetric(0.0, 0.0))
      .show(ui, |ui| {
        egui::ScrollArea::vertical().max_height(ui.available_height() - 71.0).show(ui, |ui| {
          ui.add_space(4.0);

          let mut sorted = score_ids.iter().copied().map(|id| score_chache.score_details(id)).collect::<Vec<_>>();
          sorted.sort_unstable_by_key(|b| std::cmp::Reverse(b.score_points()));

          for (i, score) in sorted.iter().copied().enumerate() {
            write!(&mut self.buffer, "{}", i + 1).unwrap();
            render_score(ui, score, &self.buffer);
            self.buffer.clear();

            ui.add_space(4.0);
          }
        });
      });
  }
}

fn render_score(ui: &mut egui::Ui, score: &Score, idx: &str) {
  ui.horizontal(|ui| {
    use egui_extras::{Size, StripBuilder};

    ui.add_space(28.0);

    StripBuilder::new(ui)
      .size(Size::exact(32.0))
      .size(Size::remainder().at_most(512.0))
      .horizontal(|mut strip| {
        strip.cell(|ui| {
          ui.vertical_centered(|ui| {
            ui.add_space(21.0);
            egui::Label::new(egui::RichText::new(idx).size(16.0).strong()).truncate().ui(ui);
          });
        });

        strip.cell(|ui| {
          egui::Frame::window(ui.style())
            .fill(egui::Color32::from_rgba_unmultiplied(20, 20, 20, 240))
            .outer_margin(egui::Margin {
              left: 6.0,
              right: 12.0,
              top: 0.0,
              bottom: 0.0,
            })
            .inner_margin(egui::Margin::same(8.0).tap_mut(|x| x.right = 12.0))
            .show(ui, |ui| {
              ui.horizontal(|ui| {
                egui::Image::new("file://assets/avatar.png")
                  .rounding(egui::Rounding::same(4.0))
                  .fit_to_exact_size(egui::vec2(48.0, 48.0))
                  .ui(ui);

                ui.vertical(|ui| {
                  ui.add_space(3.0);

                  ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(egui::RichText::new(score.username()).size(18.0).strong());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                      let grade = score.grade();
                      let color = grade.color();
                      ui.label(egui::RichText::new(format!("{}", grade)).size(18.0).color(color));

                      ui.label(egui::RichText::new(format!("{}", score.score_points())).size(18.0).strong());
                    });
                  });

                  ui.add_space(1.0);
                  ui.horizontal(|ui| {
                    let fmt = timeago::Formatter::new();
                    let date = Timestamp::now().since(score.date()).unwrap();
                    let text = fmt.convert(date.try_into().unwrap());

                    ui.label(egui::RichText::new(text).weak().size(14.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                      ui.label(egui::RichText::new(format!("{}x", score.max_combo())).size(14.0));
                      ui.label(egui::RichText::new("∙").size(14.0));
                      ui.label(egui::RichText::new(format!("{:.2}%", score.accuracy() * 100.0)).size(14.0));
                    });
                  });
                });
              });
            });
        });
      });
  });
}
