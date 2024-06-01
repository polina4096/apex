use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum ClientEvent {
  RetryBeatmap,
  ToggleSettings,
  SelectBeatmap {
    path: PathBuf,
  },
}
