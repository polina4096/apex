use self::{input_state::InputState, keybinds::Keybinds};

pub mod action;
pub mod input_state;
pub mod keybinds;

#[rustfmt::skip]
pub struct Input<T> {
  pub state    : InputState,
  pub keybinds : Keybinds<T>,
  pub grabbing : bool,
}

impl<T> Default for Input<T> {
  fn default() -> Self {
    #[rustfmt::skip] return Self {
      state    : InputState::default(),
      keybinds : Keybinds::default(),
      grabbing : false,
    };
  }
}
