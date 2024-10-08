use std::fmt::Write as _;

use apex_framework::event::EventBus;
use egui::Widget;
use jiff::Timestamp;
use tap::Tap as _;

use crate::client::{
  event::ClientEvent,
  gameplay::beatmap::BeatmapHash,
  score::{
    score::Score,
    score_cache::{ScoreCache, ScoreId},
  },
};

pub struct BeatmapScores {
  event_bus: EventBus<ClientEvent>,

  buffer: String,
}

impl BeatmapScores {
  pub fn new(event_bus: EventBus<ClientEvent>) -> Self {
    return Self { event_bus, buffer: String::new() };
  }

  pub fn prepare(
    &mut self,
    ui: &mut egui::Ui,
    score_ids: &[ScoreId],
    score_cache: &ScoreCache,
    beatmap_hash: BeatmapHash,
  ) {
    let color = egui::Color32::from_black_alpha(160);
    let rect = ui.cursor().tap_mut(|rect| {
      rect.min.y -= 2.0;
      rect.set_height(ui.available_height() - 70.0);
      rect.set_width(ui.available_width().min(512.0 + 32.0 + 24.0 + 12.0));

      // Position it a bit inside
      rect.min.y += 16.0;
      rect.max.y += 16.0;
    });

    ui.painter().rect(rect, egui::Rounding::ZERO, color, egui::Stroke::NONE);

    egui::Frame::window(ui.style())
      .inner_margin(egui::Margin::symmetric(12.0, 8.0).tap_mut(|x| x.right += 12.0))
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

    egui::Frame::none().inner_margin(egui::Margin::ZERO).show(ui, |ui| {
      ui.style_mut().spacing.scroll.bar_width = 4.0;
      egui::ScrollArea::vertical().max_height(ui.available_height() - 75.0).show(ui, |ui| {
        ui.add_space(4.0);

        let mut sorted = score_ids.iter().copied().map(|id| (id, score_cache.score_details(id))).collect::<Vec<_>>();
        sorted.sort_unstable_by_key(|b| std::cmp::Reverse(b.1.score_points()));

        if sorted.is_empty() {
          let rect = ui.max_rect().tap_mut(|rect| {
            rect.set_width(ui.available_width().min(512.0 + 32.0 + 24.0 + 12.0));
          });

          let text = ui.painter().layout_no_wrap(
            String::from("No scores yet!"),
            egui::FontId::proportional(16.0),
            egui::Color32::PLACEHOLDER,
          );

          let pos = (rect.center() - (text.size() / 2.0)).tap_mut(|p| p.x += 6.0);
          ui.painter().galley(pos, text, ui.style().visuals.text_color());

          return;
        }

        for (i, (score_id, score)) in sorted.iter().copied().enumerate() {
          write!(&mut self.buffer, "{}", i + 1).unwrap();
          if render_score(ui, score, &self.buffer).clicked() {
            self.event_bus.send(ClientEvent::ViewScore { beatmap_hash, score_id: score_id });
          }
          self.buffer.clear();

          ui.add_space(4.0);
        }
      });
    });
  }
}

fn render_score(ui: &mut egui::Ui, score: &Score, idx: &str) -> egui::Response {
  let mut rect = ui.cursor();
  rect.set_width(ui.available_width().min(512.0) - 12.0);
  rect.set_height(48.0 + 16.0);
  rect = rect.translate(egui::vec2(32.0 + 28.0 + 6.0 + 3.0, 4.0));

  ui.push_id(idx, |ui| {
    let hovered = ui.rect_contains_pointer(rect);

    ui.horizontal(|ui| {
      use egui_extras::{Size, StripBuilder};

      StripBuilder::new(ui)
        .size(Size::exact(60.0))
        .size(Size::remainder().at_most(512.0))
        .horizontal(|mut strip| {
          strip.cell(|ui| {
            let rect = ui.max_rect().tap_mut(|rect| {
              rect.set_height(48.0 + 16.0);
            });

            let text =
              ui.painter()
                .layout_no_wrap(idx.to_owned(), egui::FontId::proportional(16.0), egui::Color32::PLACEHOLDER);

            let pos = (rect.center() - (text.size() / 2.0)).tap_mut(|p| p.x += 6.0);
            ui.painter().galley(pos, text, ui.style().visuals.strong_text_color());
          });

          strip.cell(|ui| {
            let inactive = egui::Color32::from_rgba_unmultiplied(20, 20, 20, 240);
            let active = egui::Color32::from_rgba_unmultiplied(30, 30, 30, 245);
            egui::Frame::window(ui.style())
              .fill(if hovered { active } else { inactive })
              .outer_margin(egui::Margin {
                left: 0.0,
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

                      ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
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
  });

  return ui
    .interact(rect, egui::Id::new(idx).with("score"), egui::Sense::click())
    .on_hover_cursor(egui::CursorIcon::PointingHand)
    .interact(egui::Sense::click());
}
