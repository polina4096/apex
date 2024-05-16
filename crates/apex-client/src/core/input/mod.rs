use self::{bind::KeybindManager, input_state::InputState};

pub mod input_state;
pub mod bind;

pub struct Input<T> {
  pub state    : InputState,
  pub keybinds : KeybindManager<T>,
  pub grabbing : bool,
}

impl<T> Default for Input<T> {
  fn default() -> Self {
    return Self {
      state    : InputState::default(),
      keybinds : KeybindManager::default(),
      grabbing : false,
    };
  }
}
