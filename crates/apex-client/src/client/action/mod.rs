use actions::{
  back::Back,
  next::Next,
  prev::Prev,
  recording::Recording,
  retry::Retry,
  select::Select,
  settings::Settings,
  taiko::{DonOne, DonTwo, KatOne, KatTwo},
};

use crate::actions;

use super::client::Client;

pub mod action;
pub mod actions;

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
