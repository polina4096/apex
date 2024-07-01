use std::path::PathBuf;

use action_bar::ActionBar;
use beatmap_card::BeatmapCard;
use beatmap_list::BeatmapList;
use beatmap_stats::BeatmapStats;

use crate::{
  client::{
    client::Client,
    event::ClientEvent,
    gameplay::{beatmap_cache::BeatmapCache, beatmap_selector::BeatmapSelector},
  },
  core::{core::Core, event::EventBus, time::clock::AbstractClock},
};

use super::background_component::BackgroundComponent;

pub mod action_bar;
pub mod beatmap_card;
pub mod beatmap_list;
pub mod beatmap_stats;

pub struct BeatmapSelectionView {
  event_bus: EventBus<ClientEvent>,

  prev_beatmap: PathBuf,

  beatmap_bg: BackgroundComponent,
  beatmap_list: BeatmapList,
  beatmap_stats: BeatmapStats,
  action_bar: ActionBar,
}

impl BeatmapSelectionView {
  pub fn new(event_bus: EventBus<ClientEvent>, beatmap_cache: &BeatmapCache, clock: &mut impl AbstractClock) -> Self {
    let mut beatmap_cards = vec![];
    for (path, info) in beatmap_cache.iter() {
      let card = BeatmapCard::new(path, info);
      beatmap_cards.push(card);
    }

    return Self {
      event_bus: event_bus.clone(),

      prev_beatmap: PathBuf::new(),

      beatmap_bg: BackgroundComponent::new(""),
      beatmap_list: BeatmapList::new(event_bus.clone(), beatmap_cards),
      beatmap_stats: BeatmapStats::new(event_bus.clone()),
      action_bar: ActionBar::new(event_bus, clock),
    };
  }

  pub fn prepare(
    &mut self,
    core: &mut Core<Client>,
    beatmap_cache: &BeatmapCache,
    selector: &mut BeatmapSelector,
    clock: &mut impl AbstractClock,
  ) {
    selector.tick(beatmap_cache);

    use egui_extras::{Size, StripBuilder};

    let selected = selector.selected();
    let Some((path, info)) = beatmap_cache.get_index(selected) else {
      // TODO: Show error message no beatmaps found
      return;
    };

    if self.prev_beatmap != *path {
      self.prev_beatmap = path.clone();

      let bg_path = path.parent().unwrap().join(&info.bg_path);
      let bg = format!("file://{}", bg_path.to_str().unwrap());
      self.beatmap_bg = BackgroundComponent::new(bg);
    }

    egui::CentralPanel::default().frame(egui::Frame::none()).show(core.egui_ctx(), |ui| {
      self.beatmap_bg.prepare(ui);

      StripBuilder::new(ui).size(Size::remainder()).size(Size::relative(0.4)).horizontal(|mut builder| {
        builder.cell(|ui| {
          // let max_width = ui.available_width();
          // ui.set_width(max_width.min(640.0));

          ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            self.beatmap_stats.prepare(ui, info);

            // egui::Frame::window(ui.style())
            //   .outer_margin(egui::Margin::same(12.0))
            //   .inner_margin(egui::Margin::symmetric(24.0, 16.0))
            //   .show(ui, |ui| {
            //     ui.set_max_height(0.0);

            //     ui.label("text");
            //   });
          });

          ui.with_layout(egui::Layout::left_to_right(egui::Align::Max), |ui| {
            self.action_bar.prepare(ui, clock);
          });
        });

        builder.cell(|ui| {
          self.beatmap_list.prepare(ui, beatmap_cache, selector);
        });
      });
    });
  }
}
