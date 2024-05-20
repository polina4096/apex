use std::sync::Arc;

use egui::Widget;
use nucleo::{pattern::{CaseMatching, Normalization}, Nucleo};

use crate::{client::{client::Client, event::ClientEvent, gameplay::beatmap_cache::BeatmapCache}, core::{core::Core, event::EventBus}};

use super::beatmap_card::BeatmapCard;

pub struct BeatmapListView {
  event_bus: EventBus<ClientEvent>,

  matcher: Nucleo<(usize, String)>,

  search_query: String,

  beatmap_cards: Vec<BeatmapCard>,

  previous_idx: usize,
  selected_idx: usize,
}

impl BeatmapListView {
  pub fn new(event_bus: EventBus<ClientEvent>, beatmap_cache: &BeatmapCache) -> Self {
    let matcher = Nucleo::new(
      nucleo::Config::DEFAULT,
      Arc::new(|| {}),
      std::thread::available_parallelism().map(|x| x.get()).ok(),
      1,
    );

    let mut beatmap_cards = vec![];
    for (i, (path, info)) in beatmap_cache.iter().enumerate() {
      let card = BeatmapCard::new(path.clone(), info.clone());
      beatmap_cards.push(card);

      let q_str = format!("{}{}{}{}", &info.title, &info.artist, &info.difficulty, &info.creator);
      matcher.injector().push((i, q_str), |(_, q_str), cols| {
        cols[0] = q_str.clone().into();
      });
    }

    return Self {
      event_bus,
      matcher,
      search_query: String::new(),
      beatmap_cards,
      previous_idx: 0,
      selected_idx: 0,
    };
  }

  pub fn prepare(&mut self, core: &mut Core<Client>) {
    self.matcher.tick(10);

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
                .fill(ui.style().visuals.window_fill)
                .stroke(egui::Stroke::new(2.0, ui.style().visuals.window_stroke.color))
                .inner_margin(egui::Margin::same(0.0))
                .outer_margin(egui::Margin {
                  right: 6.0,
                  top: 6.0,
                  bottom: 6.0,
                  .. Default::default()
                })
                .show(ui, |ui| {
                  ui.style_mut().text_styles.iter_mut().for_each(|s| s.1.size = 16.0);
                  egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("type to search...")
                    .desired_width(f32::INFINITY)
                    .interactive(false)
                    .frame(false)
                    .ui(ui);
                });

              egui::ScrollArea::vertical().show(ui, |ui| {
                let snapshot = self.matcher.snapshot().matched_items(..);
                for result in snapshot {
                  let orig_idx = result.data.0;
                  let card = &mut self.beatmap_cards[orig_idx];

                  ui.push_id(orig_idx, |ui| {
                    let is_selected = orig_idx == self.selected_idx;
                    let response = card.prepare(ui, is_selected);

                    if is_selected && self.previous_idx != self.selected_idx {
                      self.previous_idx = self.selected_idx;
                      response.scroll_to_me(Some(egui::Align::Center));
                    }

                    if response.interact(egui::Sense::click()).clicked() {
                      self.selected_idx = orig_idx;
                      self.previous_idx = orig_idx;

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

  pub fn clear_search_query(&mut self) {
    self.search_query.clear();
    self.matcher.pattern.reparse(0, &self.search_query, CaseMatching::Ignore, Normalization::Smart, false);
  }

  pub fn has_search_query(&self) -> bool {
    return !self.search_query.is_empty();
  }

  pub fn append_search_query(&mut self, c: char) {
    self.search_query.push(c);
    self.matcher.pattern.reparse(0, &self.search_query, CaseMatching::Ignore, Normalization::Smart, false);
  }

  pub fn pop_search_query(&mut self) {
    self.search_query.pop();
    self.matcher.pattern.reparse(0, &self.search_query, CaseMatching::Ignore, Normalization::Smart, false);
  }

  // TODO: DRY refactor whatever
  pub fn select_next(&mut self) {
    let snapshot = self.matcher.snapshot();
    let mut iter = snapshot.matched_items(..);
    let Some(idx) = iter.position(|x| x.data.0 == self.selected_idx) else {
      self.selected_idx = snapshot.matched_items(..).next().map(|x| x.data.0).unwrap_or(0);
      return;
    };

    if let Some(info) = snapshot.get_matched_item(idx as u32 + 1) {
      self.selected_idx = info.data.0;
    }
  }

  pub fn select_prev(&mut self) {
    let snapshot = self.matcher.snapshot();
    let mut iter = snapshot.matched_items(..);
    let Some(idx) = iter.position(|x| x.data.0 == self.selected_idx) else {
      self.selected_idx = snapshot.matched_items(..).next().map(|x| x.data.0).unwrap_or(0);
      return;
    };

    if let Some(info) = snapshot.get_matched_item(idx as u32 - 1) {
      self.selected_idx = info.data.0;
    }
  }

  pub fn select(&mut self) {
    let path = self.beatmap_cards[self.selected_idx].path.clone();
    self.event_bus.send(ClientEvent::SelectBeatmap { path });
  }
}
