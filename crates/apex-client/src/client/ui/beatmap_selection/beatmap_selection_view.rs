use crate::{client::{event::ClientEvent, gameplay::beatmap_cache::BeatmapCache, util::beatmap_selector::BeatmapSelector}, core::event::EventBus};

use super::{beatmap_background::BeatmapBackground, beatmap_card::BeatmapCard, beatmap_list::BeatmapList, beatmap_stats::BeatmapStats};

pub struct BeatmapSelectionView {
  beatmap_bg: BeatmapBackground,
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
      beatmap_bg: BeatmapBackground::new(),
      beatmap_stats: BeatmapStats::new(),
      beatmap_list: BeatmapList::new(event_bus, beatmap_cards),
    };
  }

  pub fn prepare(&mut self, ctx: &egui::Context, beatmap_cache: &BeatmapCache, selector: &mut BeatmapSelector) {
    selector.tick();

    use egui_extras::{StripBuilder, Size};

    let selected = selector.selected();
    let Some((path, info)) = beatmap_cache.get_index(selected) else { panic!() };

    egui::CentralPanel::default()
      .frame(egui::Frame::none())
      .show(ctx, |ui| {
        let bg_path = path.parent().unwrap().join(&info.bg_path);
        self.beatmap_bg.prepare(ui, &bg_path);

        StripBuilder::new(ui)
          .size(Size::remainder())
          .size(Size::relative(0.4))
          .horizontal(|mut builder| {
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
