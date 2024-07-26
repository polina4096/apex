use back::Back;
use next::Next;
use prev::Prev;
use recording::Recording;
use retry::Retry;
use select::Select;
use settings::Settings;
use taiko::{DonOne, DonTwo, KatOne, KatTwo};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

use crate::{actions, core::input::keybinds::KeyCombination};

use super::client::Client;

pub mod back;
pub mod next;
pub mod prev;
pub mod recording;
pub mod retry;
pub mod select;
pub mod settings;
pub mod taiko;

actions! {
  ClientAction<Client> {
    /// Return to the previous state
    Back = KeyCombination::new(PhysicalKey::Code(KeyCode::Escape), ModifiersState::empty()),
    /// Open settings menu
    Settings = KeyCombination::new(PhysicalKey::Code(KeyCode::Comma), ModifiersState::SUPER),
    /// Open recording menu
    Recording = KeyCombination::new(PhysicalKey::Code(KeyCode::KeyR), ModifiersState::SUPER),

    /// Select next element
    Next = KeyCombination::new(PhysicalKey::Code(KeyCode::ArrowDown), ModifiersState::empty()),
    /// Select previous element
    Prev = KeyCombination::new(PhysicalKey::Code(KeyCode::ArrowUp), ModifiersState::empty()),

    /// Pick selected element
    Select = KeyCombination::new(PhysicalKey::Code(KeyCode::Enter), ModifiersState::empty()),
    /// Replay a beatmap from the beginning
    Retry = KeyCombination::new(PhysicalKey::Code(KeyCode::Backquote), ModifiersState::empty()),

    /// Kat (blue)
    KatOne as "Kat 1" = KeyCombination::new(PhysicalKey::Code(KeyCode::KeyS), ModifiersState::empty()),
    /// Don (red)
    DonOne as "Don 1" = KeyCombination::new(PhysicalKey::Code(KeyCode::KeyL), ModifiersState::empty()),
    /// Kat (blue)
    KatTwo as "Kat 2" = KeyCombination::new(PhysicalKey::Code(KeyCode::KeyD), ModifiersState::empty()),
    /// Don (red)
    DonTwo as "Don 2" = KeyCombination::new(PhysicalKey::Code(KeyCode::KeyK), ModifiersState::empty()),
  }
}
