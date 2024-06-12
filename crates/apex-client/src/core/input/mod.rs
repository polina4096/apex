use self::{bind::KeybindManager, input_state::InputState};

pub mod bind;
pub mod input_state;

#[rustfmt::skip]
pub struct Input<T> {
  pub state    : InputState,
  pub keybinds : KeybindManager<T>,
  pub grabbing : bool,
}

impl<T> Default for Input<T> {
  fn default() -> Self {
    #[rustfmt::skip] return Self {
      state    : InputState::default(),
      keybinds : KeybindManager::default(),
      grabbing : false,
    };
  }
}
