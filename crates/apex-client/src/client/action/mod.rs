use back::Back;
use next::Next;
use prev::Prev;
use recording::Recording;
use retry::Retry;
use select::Select;
use settings::Settings;
use taiko::{DonOne, DonTwo, KatOne, KatTwo};

use crate::actions;

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
    Back,
    Settings,
    Recording,

    Next,
    Prev,

    Select,
    Retry,

    KatOne,
    DonOne,
    KatTwo,
    DonTwo,
  }
}
