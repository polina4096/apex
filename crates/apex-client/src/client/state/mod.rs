use gameplay_state::GameplayState;
use serde::{Deserialize, Serialize};
use taiko_state::TaikoState;

pub mod taiko_state;
pub mod gameplay_state;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
  pub taiko: TaikoState,
  pub gameplay: GameplayState,
}

impl Default for GameState {
  fn default() -> Self {
    return Self {
      taiko: TaikoState::default(),
      gameplay: GameplayState::default(),
    };
  }
}
