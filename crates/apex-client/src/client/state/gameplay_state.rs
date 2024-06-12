use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayState {
  /// Offset of the audio in milliseconds
  pub audio_offset: i32,
}

impl Default for GameplayState {
  fn default() -> Self {
    return Self { audio_offset: 0 };
  }
}
