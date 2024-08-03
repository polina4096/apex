use std::path::PathBuf;

use super::score::{score::Score, score_cache::ScoreId};

#[derive(Debug)]
pub enum ClientEvent {
  RetryBeatmap,
  ToggleSettings,
  ToggleRecordingWindow,
  ShowResultScreen { path: PathBuf, score: Score },
  ViewScore { path: PathBuf, score_id: ScoreId },
  PickBeatmap { path: PathBuf },
  SelectBeatmap,
}
