use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum ClientEvent {
  RetryBeatmap,
  ToggleSettings,
  SelectBeatmap {
    path: PathBuf,
  },

  // TODO: this probably shouldn't be here
  RebuildTaikoRendererInstances,
}
