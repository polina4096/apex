use crate::{
  client::{
    client::Client, event::ClientEvent, gameplay::beatmap_cache::BeatmapCache,
    ui::beatmap_selection::BeatmapSelectionView, util::beatmap_selector::BeatmapSelector,
  },
  core::{core::Core, event::EventBus},
};

pub struct ResultScreen {
  // beatmap_selection: BeatmapSelectionView,
}

impl ResultScreen {
  pub fn new(event_bus: EventBus<ClientEvent>, beatmap_cache: &BeatmapCache) -> Self {
    return Self {};
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, beatmap_cache: &BeatmapCache) {
    // self.beatmap_selection.prepare(core, beatmap_cache, &mut self.beatmap_selector);
  }
}
