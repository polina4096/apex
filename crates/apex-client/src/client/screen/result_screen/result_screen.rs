use apex_framework::core::Core;

use crate::client::{
  client::Client, gameplay::beatmap::Beatmap, score::score::Score, ui::play_results::PlayResultsView,
};

pub struct ResultScreen {
  play_results: PlayResultsView,
}

impl ResultScreen {
  pub fn new() -> Self {
    let play_results = PlayResultsView::new("", Beatmap::default(), Score::default());

    return Self { play_results };
  }

  pub fn set_score(&mut self, beatmap: Beatmap, score: Score) {
    let bg = beatmap.file_path.parent().unwrap().join(&beatmap.bg_path);
    let bg = format!("file://{}", bg.to_str().unwrap());

    self.play_results = PlayResultsView::new(bg, beatmap, score);
  }

  pub fn prepare(&mut self, core: &mut Core<Client>) {
    self.play_results.prepare(core);
  }
}
