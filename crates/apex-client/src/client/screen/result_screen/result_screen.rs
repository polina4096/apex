use std::path::Path;

use crate::{
  client::{
    client::Client, event::ClientEvent, gameplay::beatmap_cache::BeatmapCache, state::AppState,
    ui::play_results::PlayResultsView,
  },
  core::{core::Core, event::EventBus},
};

pub struct ResultScreen {
  play_results: PlayResultsView,
}

impl ResultScreen {
  pub fn new(event_bus: EventBus<ClientEvent>, beatmap_cache: &BeatmapCache, beatmap: &Path) -> Self {
    let play_results = PlayResultsView::new("");

    return Self { play_results };
  }

  pub fn finish(&mut self, beatmap_cache: &BeatmapCache, path: &Path) {
    let bg = beatmap_cache.get(path).map(|x| path.parent().unwrap().join(&x.bg_path)).unwrap_or_default();
    let bg = format!("file://{}", bg.to_str().unwrap());

    self.play_results = PlayResultsView::new(bg);
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, state: &mut AppState, beatmap_cache: &BeatmapCache) {
    self.play_results.prepare(core, state);
  }
}
