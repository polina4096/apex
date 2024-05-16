use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum ClientEvent {
  SelectBeatmap {
    path: PathBuf,
  },
}
