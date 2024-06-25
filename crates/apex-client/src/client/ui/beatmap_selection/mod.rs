use std::path::PathBuf;

use beatmap_card::BeatmapCard;
use beatmap_list::BeatmapList;
use beatmap_stats::BeatmapStats;

use crate::{
  client::{
    client::Client,
    event::ClientEvent,
    gameplay::{beatmap_cache::BeatmapCache, beatmap_selector::BeatmapSelector},
  },
  core::{core::Core, event::EventBus},
};

use super::background_component::BackgroundComponent;

pub mod beatmap_card;
pub mod beatmap_list;
pub mod beatmap_stats;

pub struct BeatmapSelectionView {
  prev_beatmap: PathBuf,

  beatmap_bg: BackgroundComponent,
  beatmap_stats: BeatmapStats,
  beatmap_list: BeatmapList,
}

impl BeatmapSelectionView {
  pub fn new(event_bus: EventBus<ClientEvent>, beatmap_cache: &BeatmapCache) -> Self {
    let mut beatmap_cards = vec![];
    for (path, info) in beatmap_cache.iter() {
      let card = BeatmapCard::new(path, info);
      beatmap_cards.push(card);
    }

    return Self {
      prev_beatmap: PathBuf::new(),

      beatmap_bg: BackgroundComponent::new(""),
      beatmap_stats: BeatmapStats::new(event_bus.clone()),
      beatmap_list: BeatmapList::new(event_bus, beatmap_cards),
    };
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, beatmap_cache: &BeatmapCache, selector: &mut BeatmapSelector) {
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
          self.beatmap_stats.prepare(ui, info);
        });

        builder.cell(|ui| {
          self.beatmap_list.prepare(ui, beatmap_cache, selector);
        });
      });
    });
  }
}
