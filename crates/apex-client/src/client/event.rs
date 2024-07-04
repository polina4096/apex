use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum ClientEvent {
  RetryBeatmap,
  ToggleSettings,
  SyncSettings,
  OpenRecordingWindow { path: PathBuf },
  ShowResultScreen { path: PathBuf },
  PickBeatmap { path: PathBuf },
  SelectBeatmap,
}
