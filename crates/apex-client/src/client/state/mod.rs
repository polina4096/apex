use gameplay_state::GameplayState;
use graphics_state::GraphicsState;
use serde::{Deserialize, Serialize};
use taiko_state::TaikoState;

pub mod taiko_state;
pub mod gameplay_state;
pub mod graphics_state;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
  pub taiko: TaikoState,
  pub gameplay: GameplayState,
  pub graphics: GraphicsState,
}

impl Default for AppState {
  fn default() -> Self {
    return Self {
      taiko: TaikoState::default(),
      gameplay: GameplayState::default(),
      graphics: GraphicsState::default(),
    };
  }
}
