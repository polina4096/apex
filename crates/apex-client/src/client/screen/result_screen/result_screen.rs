use std::path::Path;

use crate::{
  client::{
    client::Client,
    event::ClientEvent,
    gameplay::beatmap_cache::{BeatmapCache, BeatmapInfo},
    score::score_cache::{ScoreCache, ScoreId},
    ui::play_results::PlayResultsView,
  },
  core::{core::Core, event::EventBus},
};

pub struct ResultScreen {
  play_results: PlayResultsView,
}

impl ResultScreen {
  pub fn new(_event_bus: EventBus<ClientEvent>) -> Self {
    let play_results = PlayResultsView::new("", BeatmapInfo::default(), ScoreId::default());

    return Self { play_results };
  }

  pub fn set_score(&mut self, beatmap_cache: &BeatmapCache, path: &Path, score: ScoreId) {
    let Some(beatmap) = beatmap_cache.get(path) else {
      return;
    };

    let bg = path.parent().unwrap().join(&beatmap.bg_path);
    let bg = format!("file://{}", bg.to_str().unwrap());

    self.play_results = PlayResultsView::new(bg, beatmap.clone(), score);
  }

  pub fn prepare(&mut self, core: &mut Core<Client>, _beatmap_cache: &BeatmapCache, score_cache: &ScoreCache) {
    self.play_results.prepare(core, score_cache);
  }
}
