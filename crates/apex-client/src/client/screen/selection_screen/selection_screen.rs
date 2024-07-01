use crate::{
  client::{
    client::Client,
    event::ClientEvent,
    gameplay::{beatmap_cache::BeatmapCache, beatmap_selector::BeatmapSelector},
    ui::beatmap_selection::BeatmapSelectionView,
  },
  core::{core::Core, event::EventBus, time::clock::AbstractClock},
};

pub struct SelectionScreen {
  beatmap_selection: BeatmapSelectionView,
  beatmap_selector: BeatmapSelector,
}

impl SelectionScreen {
  pub fn new(event_bus: EventBus<ClientEvent>, beatmap_cache: &BeatmapCache, clock: &mut impl AbstractClock) -> Self {
    let beatmap_selection = BeatmapSelectionView::new(event_bus, beatmap_cache, clock);
    let beatmap_selector = BeatmapSelector::new(beatmap_cache);

    return Self { beatmap_selection, beatmap_selector };
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, beatmap_cache: &BeatmapCache, clock: &mut impl AbstractClock) {
    self.beatmap_selection.prepare(core, beatmap_cache, &mut self.beatmap_selector, clock);
  }

  pub fn beatmap_selector(&self) -> &BeatmapSelector {
    return &self.beatmap_selector;
  }

  pub fn beatmap_selector_mut(&mut self) -> &mut BeatmapSelector {
    return &mut self.beatmap_selector;
  }
}
