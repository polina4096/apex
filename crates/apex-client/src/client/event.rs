use std::path::PathBuf;

use super::score::score::Score;

#[derive(Debug)]
pub enum ClientEvent {
  RetryBeatmap,
  ToggleSettings,
  ToggleRecordingWindow,
  ShowResultScreen { path: PathBuf, score: Score },
  PickBeatmap { path: PathBuf },
  SelectBeatmap,
}
