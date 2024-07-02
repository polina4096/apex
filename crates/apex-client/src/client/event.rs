use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum ClientEvent {
  RetryBeatmap,
  ToggleSettings,
  ShowResultScreen { path: PathBuf },
  PickBeatmap { path: PathBuf },
  SelectBeatmap,
}
