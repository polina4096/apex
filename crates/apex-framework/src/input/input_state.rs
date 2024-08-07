use winit::keyboard::{ModifiersState, NativeKeyCode, PhysicalKey};

pub struct InputState {
  /// The last key combination that was pressed.
  pub last_pressed: PhysicalKey,

  /// The current state of the keyboard modifiers.
  pub modifiers: ModifiersState,
}

impl Default for InputState {
  fn default() -> Self {
    return Self {
      last_pressed: PhysicalKey::Unidentified(NativeKeyCode::Unidentified),
      modifiers: ModifiersState::empty(),
    };
  }
}
