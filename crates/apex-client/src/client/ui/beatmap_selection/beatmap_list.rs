use apex_framework::event::EventBus;
use egui::{NumExt as _, Widget};
use instant::Instant;
use log::debug;

use crate::client::{
  event::ClientEvent,
  gameplay::{beatmap_cache::BeatmapCache, beatmap_selector::BeatmapSelector},
};

use super::beatmap_card::BeatmapCard;

pub struct BeatmapList {
  event_bus: EventBus<ClientEvent>,
  beatmap_cards: Vec<BeatmapCard>,
  prev_selected: usize,
  scroll_to_selected: bool,

  last_update: Instant,
}

impl BeatmapList {
  pub fn new(event_bus: EventBus<ClientEvent>, beatmap_cards: Vec<BeatmapCard>) -> Self {
    return Self {
      event_bus,
      beatmap_cards,
      prev_selected: 0,
      scroll_to_selected: false,
      last_update: Instant::now(),
    };
  }

  pub fn scroll_to_selected(&mut self) {
    self.scroll_to_selected = true;
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui, beatmap_cache: &BeatmapCache, selector: &mut BeatmapSelector) {
    // TODO: this is going to be very slow on a large number of beatmaps, probably go with event based approach
    if beatmap_cache.last_update() > self.last_update {
      debug!("Updating beatmap list");

      self.last_update = Instant::now();
      self.beatmap_cards.clear();

      for (_, info) in beatmap_cache.iter() {
        let card = BeatmapCard::new(info);
        self.beatmap_cards.push(card);
      }
    }

    egui::Frame::none()
      .fill(egui::Color32::from_black_alpha(128))
      .outer_margin(egui::Margin { left: -9.0, ..Default::default() })
      .inner_margin(egui::Margin {
        left: 12.0,
        right: 12.0,
        ..Default::default()
      })
      .show(ui, |ui| {
        ui.set_height(ui.available_height());
        ui.set_width(ui.available_width());

        egui::Window::new("search_bar_window")
          .frame(
            egui::Frame::window(ui.style()) //
              .inner_margin(egui::Margin::symmetric(8.0, 6.0))
              .outer_margin(egui::Margin {
                top: 8.0,
                bottom: 8.0,
                right: 11.75,
                ..Default::default()
              }),
          )
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

        let total_rows = selector.matched().count();
        let row_height_sans_spacing = 72.0;

        let spacing = ui.spacing().item_spacing;
        let row_height_with_spacing = row_height_sans_spacing + spacing.y;

        egui::ScrollArea::vertical()
          .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
          .show_viewport(ui, |ui, viewport| {
            ui.set_height((row_height_with_spacing * total_rows as f32 - spacing.y).at_least(0.0));

            let mut min_row = (viewport.min.y / row_height_with_spacing).floor() as usize;
            let mut max_row = (viewport.max.y / row_height_with_spacing).ceil() as usize + 1;
            if max_row > total_rows {
              let diff = max_row.saturating_sub(min_row);
              max_row = total_rows;
              min_row = total_rows.saturating_sub(diff);
            }

            let y_min = ui.max_rect().top() + min_row as f32 * row_height_with_spacing;
            let y_max = ui.max_rect().top() + max_row as f32 * row_height_with_spacing;

            let rect = egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), y_min - 54.0 ..= y_max);

            ui.allocate_ui_at_rect(rect, |ui| {
              ui.skip_ahead_auto_ids(min_row);

              if min_row == 0 {
                ui.add_space(54.0 + 54.0);
              } else {
                ui.add_space(32.0);
              }

              let mut new_selected = None;
              let selected_idx = selector.selected();

              for orig_idx in selector.matched().skip(if min_row == 0 { min_row } else { min_row - 1 }).take(max_row) {
                let card = &mut self.beatmap_cards[orig_idx];
                let (beatmap_hash, _) = beatmap_cache.get_index(orig_idx).unwrap();

                ui.push_id(orig_idx, |ui| {
                  let is_selected = orig_idx == selected_idx;
                  let response = card.prepare(ui, is_selected, beatmap_hash, &self.event_bus);
                  let sense = response.interact(egui::Sense::click());

                  let clicked_secondary = sense.clicked_by(egui::PointerButton::Secondary);

                  if clicked_secondary {
                    self.prev_selected = orig_idx;
                  }

                  if sense.clicked() || clicked_secondary {
                    new_selected = Some(orig_idx);

                    self.event_bus.send(ClientEvent::SelectBeatmap);

                    if is_selected && !clicked_secondary {
                      self.event_bus.send(ClientEvent::PickBeatmap { beatmap_hash });
                    }
                  }
                });
              }

              if let Some(new_selected) = new_selected {
                selector.set_selected(new_selected);
              }

              let selected_idx = selector.selected();
              if self.prev_selected != selected_idx || self.scroll_to_selected {
                self.prev_selected = selected_idx;
                if self.scroll_to_selected {
                  self.scroll_to_selected = false;
                }

                let mut height = 0.0;
                for idx in selector.matched() {
                  if idx == selected_idx {
                    break;
                  }

                  height += row_height_with_spacing;
                }

                let screen_center_offset_y = viewport.height() / 2.0;
                let delta_y = viewport.top() - height + screen_center_offset_y - row_height_with_spacing;

                ui.scroll_with_delta(egui::vec2(0.0, delta_y));
              }

              ui.add_space(4.0);
            });
          });
      });
  }
}
