use back::Back;
use clear_query::ClearQuery;
use debug::Debug;
use next::Next;
use prev::Prev;
use recording::Recording;
use retry::Retry;
use select::Select;
use settings::Settings;
use skip::Skip;
use taiko::{DonLeft, DonRight, KatLeft, KatRight};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

use apex_framework::{actions, input::keybinds::KeyCombination, key_comb};

use super::client::Client;

pub mod back;
pub mod clear_query;
pub mod debug;
pub mod next;
pub mod prev;
pub mod recording;
pub mod retry;
pub mod select;
pub mod settings;
pub mod skip;
pub mod taiko;

actions! {
  ClientAction<Client> {
    /// Return to the previous state
    Back = key_comb!(Escape),

    /// Clear selection query
    ClearQuery = key_comb!(Ctrl + Backspace),

    /// Open settings menu
    Settings = key_comb!(Super + Comma),
    /// Open recording menu
    Recording = key_comb!(Super + KeyR),
    /// Open debug menu
    Debug = key_comb!(Super + F1),

    /// Select next element
    Next = key_comb!(ArrowDown),
    /// Select previous element
    Prev = key_comb!(ArrowUp),
    /// Pick selected element
    Select = key_comb!(Enter),

    /// Restart the beatmap
    Retry = key_comb!(Backquote),

    /// Skip current section
    Skip = key_comb!(Space),

    /// Kat (blue)
    KatLeft as "Kat Left" = key_comb!(KeyL),
    /// Don (red)
    DonLeft as "Don Left" = key_comb!(KeyK),
    /// Kat (blue)
    KatRight as "Kat Right" = key_comb!(KeyS),
    /// Don (red)
    DonRight as "Don Right" = key_comb!(KeyD),
  }
}
