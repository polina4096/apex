use crate::{client::{client::Client, event::ClientEvent, gameplay::beatmap_cache::BeatmapCache, ui::beatmap_selection::BeatmapSelectionView, util::beatmap_selector::BeatmapSelector}, core::{core::Core, event::EventBus}};

pub struct SelectionScreen {
  beatmap_selection: BeatmapSelectionView,
  beatmap_selector: BeatmapSelector,
}

impl SelectionScreen {
  pub fn new(event_bus: EventBus<ClientEvent>, beatmap_cache: &BeatmapCache) -> Self {
    let beatmap_selection = BeatmapSelectionView::new(event_bus, beatmap_cache);
    let beatmap_selector = BeatmapSelector::new(beatmap_cache);

    return Self {
      beatmap_selection,
      beatmap_selector,
    };
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, beatmap_cache: &BeatmapCache) {
    self.beatmap_selection.prepare(core.egui_ctx(), beatmap_cache, &mut self.beatmap_selector);
  }

  pub fn beatmap_selector(&self) -> &BeatmapSelector {
    return &self.beatmap_selector;
  }

  pub fn beatmap_selector_mut(&mut self) -> &mut BeatmapSelector {
    return &mut self.beatmap_selector;
  }
}
