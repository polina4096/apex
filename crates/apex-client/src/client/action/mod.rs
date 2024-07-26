use back::Back;
use next::Next;
use prev::Prev;
use recording::Recording;
use retry::Retry;
use select::Select;
use settings::Settings;
use taiko::{DonOne, DonTwo, KatOne, KatTwo};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

use crate::{actions, core::input::keybinds::KeyCombination, key_comb};

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
    Back = key_comb!(Escape),
    /// Open settings menu
    Settings = key_comb!(Super + Comma),
    /// Open recording menu
    Recording = key_comb!(Super + KeyR),

    /// Select next element
    Next = key_comb!(ArrowDown),
    /// Select previous element
    Prev = key_comb!(ArrowUp),

    /// Pick selected element
    Select = key_comb!(Enter),
    /// Replay a beatmap from the beginning
    Retry = key_comb!(Backquote),

    /// Kat (blue)
    KatOne as "Kat 1" = key_comb!(KeyS),
    /// Don (red)
    DonOne as "Don 1" = key_comb!(KeyL),
    /// Kat (blue)
    KatTwo as "Kat 2" = key_comb!(KeyD),
    /// Don (red)
    DonTwo as "Don 2" = key_comb!(KeyK),
  }
}
