use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum ClientEvent {
  RetryBeatmap,
  ToggleSettings,
  ShowResultScreen { path: PathBuf },
  SelectBeatmap { path: PathBuf },

  // TODO: this probably shouldn't be here
  RebuildTaikoRendererInstances,
}
