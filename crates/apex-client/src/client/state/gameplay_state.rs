use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayState {
  /// Offset of the audio in milliseconds
  pub audio_offset: i64,

  /// Additional time before the first note
  pub lead_in: u64,

  /// Additional time after the last note
  pub lead_out: u64,
}

impl Default for GameplayState {
  fn default() -> Self {
    return Self {
      audio_offset: 0,
      lead_in: 1000,
      lead_out: 1000,
    };
  }
}
