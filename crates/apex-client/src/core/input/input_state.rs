use winit::keyboard::{ModifiersState, NativeKeyCode, PhysicalKey};

use super::bind::KeyCombination;

pub struct InputState {
  pub last_comb : KeyCombination,
  pub modifiers : ModifiersState,
}

impl Default for InputState {
  fn default() -> Self {
    let key = PhysicalKey::Unidentified(NativeKeyCode::Unidentified);

    return Self {
      last_comb : KeyCombination::new(key, ModifiersState::empty()),
      modifiers : ModifiersState::empty(),
    };
  }
}
