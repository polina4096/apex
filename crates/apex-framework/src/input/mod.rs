use std::hash::Hash;

use action::AppActions;
use tap::Pipe as _;

use self::{input_state::InputState, keybinds::Keybinds};

pub mod action;
pub mod input_state;
pub mod keybinds;

#[rustfmt::skip]
pub struct Input<T> {
  pub keybinds : Keybinds<T>,
  pub state    : InputState,
  pub grabbing : bool,
}

impl<T: AppActions + Copy + Eq + Hash> Input<T> {
  pub fn with_keybinds(mut keybinds: Keybinds<T>) -> Self {
    return Self {
      keybinds: Keybinds::<T>::default().pipe(|mut binds| {
        T::insert_keybinds(&mut binds);
        keybinds.merge(binds);
        return keybinds;
      }),
      state: InputState::default(),
      grabbing: false,
    };
  }
}

impl<T> Default for Input<T> {
  fn default() -> Self {
    #[rustfmt::skip] return Self {
      keybinds : Keybinds::default(),
      state    : InputState::default(),
      grabbing : false,
    };
  }
}
