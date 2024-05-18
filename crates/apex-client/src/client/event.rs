use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum ClientEvent {
  RetryBeatmap,
  SelectBeatmap {
    path: PathBuf,
  },
}
