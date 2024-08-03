use std::path::Path;

use crate::{
  client::{
    client::Client,
    event::ClientEvent,
    gameplay::beatmap_cache::BeatmapCache,
    score::score_cache::{ScoreCache, ScoreId},
    settings::Settings,
    ui::play_results::PlayResultsView,
  },
  core::{core::Core, event::EventBus},
};

pub struct ResultScreen {
  play_results: PlayResultsView,
}

impl ResultScreen {
  pub fn new(_event_bus: EventBus<ClientEvent>, _beatmap_cache: &BeatmapCache, _beatmap: &Path) -> Self {
    let play_results = PlayResultsView::new("", ScoreId::default());

    return Self { play_results };
  }

  pub fn finish(&mut self, beatmap_cache: &BeatmapCache, path: &Path, score: ScoreId) {
    let bg = beatmap_cache.get(path).map(|x| path.parent().unwrap().join(&x.bg_path)).unwrap_or_default();
    let bg = format!("file://{}", bg.to_str().unwrap());

    self.play_results = PlayResultsView::new(bg, score);
  }

  pub fn prepare(
    &mut self,
    core: &mut Core<Client>,
    settings: &mut Settings,
    _beatmap_cache: &BeatmapCache,
    score_cache: &ScoreCache,
  ) {
    self.play_results.prepare(core, settings);
  }
}
