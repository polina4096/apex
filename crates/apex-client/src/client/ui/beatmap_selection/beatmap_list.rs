use std::hash::BuildHasherDefault;

use ahash::AHashMap;
use egui::Widget;
use instant::Instant;
use log::debug;
use nohash_hasher::{BuildNoHashHasher, NoHashHasher};

use crate::{
  client::{
    event::ClientEvent,
    gameplay::{beatmap_cache::BeatmapCache, beatmap_selector::BeatmapSelector},
  },
  core::event::EventBus,
};

use super::beatmap_card::BeatmapCard;

pub struct BeatmapList {
  event_bus: EventBus<ClientEvent>,
  beatmap_cards: Vec<BeatmapCard>,
  response_cache: AHashMap<usize, egui::Response, BuildHasherDefault<NoHashHasher<usize>>>,
  prev_selected: usize,

  last_update: Instant,
}

impl BeatmapList {
  pub fn new(event_bus: EventBus<ClientEvent>, beatmap_cards: Vec<BeatmapCard>) -> Self {
    return Self {
      event_bus,
      beatmap_cards,
      prev_selected: 0,
      response_cache: AHashMap::with_hasher(BuildNoHashHasher::<usize>::default()),
      last_update: Instant::now(),
    };
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui, beatmap_cache: &BeatmapCache, selector: &mut BeatmapSelector) {
    // TODO: this is going to be very slow on a large number of beatmaps, probably go with event based approach
    if beatmap_cache.last_update() > self.last_update {
      debug!("Updating beatmap list");

      self.last_update = Instant::now();
      self.beatmap_cards.clear();

      for (path, info) in beatmap_cache.iter() {
        let card = BeatmapCard::new(path, info);
        self.beatmap_cards.push(card);
      }
    }

    egui::Frame::none()
      .fill(egui::Color32::from_black_alpha(128))
      .inner_margin(egui::Margin {
        left: 12.0,
        right: 12.0,
        ..Default::default()
      })
      .show(ui, |ui| {
        ui.set_height(ui.available_height());
        ui.set_width(ui.available_width());

        egui::Window::new("search_bar_window")
          .frame(egui::Frame::window(ui.style()).inner_margin(egui::Margin::symmetric(8.0, 6.0)).outer_margin(
            egui::Margin {
              top: 8.0,
              bottom: 8.0,
              right: 11.75,
              ..Default::default()
            },
          ))
          .fixed_size(egui::vec2(ui.available_width() - 16.0, 0.0))
          .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::ZERO)
          .collapsible(false)
          .title_bar(false)
          .resizable(false)
          .show(ui.ctx(), |ui| {
            ui.style_mut().text_styles.iter_mut().for_each(|s| s.1.size = 16.0);
            egui::TextEdit::singleline(selector.query_mut())
              .hint_text("type to search...")
              .desired_width(f32::INFINITY)
              .interactive(false)
              .frame(false)
              .ui(ui);
          });

        egui::ScrollArea::vertical()
          .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
          .show(ui, |ui| {
            ui.add_space(54.0);

            let mut new_selected = None;
            let selected_idx = selector.selected();

            for orig_idx in selector.matched() {
              let card = &mut self.beatmap_cards[orig_idx];
              let (path, _) = beatmap_cache.get_index(orig_idx).unwrap();

              ui.push_id(orig_idx, |ui| {
                let is_selected = orig_idx == selected_idx;
                let response = card.prepare(ui, is_selected, path, &self.event_bus);
                let sense = response.interact(egui::Sense::click());
                self.response_cache.insert(orig_idx, response);

                let clicked_secondary = sense.clicked_by(egui::PointerButton::Secondary);

                if clicked_secondary {
                  self.prev_selected = orig_idx;
                }

                if sense.clicked() || clicked_secondary {
                  new_selected = Some(orig_idx);

                  self.event_bus.send(ClientEvent::SelectBeatmap);

                  if is_selected && !clicked_secondary {
                    self.event_bus.send(ClientEvent::PickBeatmap { path: path.clone() });
                  }
                }
              });
            }

            if let Some(new_selected) = new_selected {
              selector.set_selected(new_selected);
            }

            let selected_idx = selector.selected();
            if self.prev_selected != selected_idx {
              self.prev_selected = selected_idx;

              if let Some(response) = self.response_cache.get(&selected_idx) {
                response.scroll_to_me(Some(egui::Align::Center));
              }
            }

            ui.add_space(4.0);
          });
      });
  }
}
