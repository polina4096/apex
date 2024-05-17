use egui::Widget;
use tap::Tap;

use crate::{client::{client::Client, event::ClientEvent, taiko::beatmap_cache::BeatmapCache}, core::{core::Core, event::EventBus}};

use super::beatmap_card::BeatmapCard;

pub struct BeatmapListView {
  event_bus: EventBus<ClientEvent>,

  search_query: String,
  beatmap_cards: Vec<BeatmapCard>,
  previous_idx: usize,
  selected_idx: usize,
}

impl BeatmapListView {
  pub fn new(event_bus: EventBus<ClientEvent>, beatmap_cache: &BeatmapCache) -> Self {
    return Self {
      event_bus,
      search_query: String::new(),
      beatmap_cards: beatmap_cache.iter().map(|(path, info)| BeatmapCard::new(path.clone(), info.clone())).collect(),
      previous_idx: 0,
      selected_idx: 0,
    };
  }

  pub fn prepare(&mut self, core: &mut Core<Client>) {
    use egui_extras::{StripBuilder, Size};

    egui::CentralPanel::default()
      .frame(egui::Frame::none())
      .show(core.egui_ctx.egui_ctx(), |ui| {

        let selected = &self.beatmap_cards[self.selected_idx];
        let bg_path = selected.path.parent().unwrap().join(&selected.info.bg_path);
        let bg_uri = format!("file://{}", bg_path.to_str().unwrap());
        let img = egui::Image::new(bg_uri)
          .tint(egui::Color32::from_gray(80));

        {
          let mut rect = ui.available_rect_before_wrap();

          ui.set_clip_rect(rect);

          let img_size = img.load_and_calc_size(ui, egui::Vec2::INFINITY);
          if let Some(img_size) = img_size {
            let img_aspect = img_size.x / img_size.y;
            let scr_aspect = rect.width() / rect.height();

            let width = rect.height() * img_aspect;
            let height = rect.width() / img_aspect;

            if scr_aspect < img_aspect {
              rect.set_width(width);
              // rect = rect.translate(egui::vec2(-rect.width() / 2.0, 0.0));
            } else {
              rect.set_height(height);
              // rect = rect.translate(egui::vec2(0.0, -rect.height() / 2.0));
            }
          }

          img.paint_at(ui, rect);
        }

        StripBuilder::new(ui)
          .size(Size::remainder())
          .size(Size::relative(0.4))
          .horizontal(|mut builder| {
            builder.cell(|ui| {
              ui.label("gay gay homosexual gaaaaay");
            });

            builder.cell(|ui| {
              egui::Frame::none()
                .outer_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                  egui::TextEdit::singleline(&mut self.search_query)
                    .desired_width(f32::INFINITY)
                    .ui(ui);
                });

              egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, card) in self.beatmap_cards.iter_mut().enumerate() {
                  ui.push_id(i, |ui| {
                    let is_selected = i == self.selected_idx;
                    let response = card.prepare(ui, is_selected);

                    if is_selected && self.previous_idx != self.selected_idx {
                      self.previous_idx = self.selected_idx;
                      response.scroll_to_me(Some(egui::Align::Center));
                    }

                    if response.interact(egui::Sense::click()).clicked() {
                      self.selected_idx = i;
                      self.previous_idx = i;

                      if is_selected {
                        self.event_bus.send(ClientEvent::SelectBeatmap { path: card.path.clone() });
                      }
                    }
                  });
                }
              });
            });
          });
      });
  }

  pub fn select_next(&mut self) {
    if self.selected_idx < self.beatmap_cards.len() - 1 {
      self.selected_idx += 1;
    }
  }

  pub fn select_prev(&mut self) {
    if self.selected_idx > 0 {
      self.selected_idx -= 1;
    }
  }

  pub fn select(&mut self) {
    let path = self.beatmap_cards[self.selected_idx].path.clone();
    self.event_bus.send(ClientEvent::SelectBeatmap { path });
  }
}
