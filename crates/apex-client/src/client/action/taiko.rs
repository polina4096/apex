use crate::client::{
  client::{Client, GameState},
  gameplay::taiko_player::TaikoInput,
};

use apex_framework::core::Core;
use apex_framework::input::action::Action;

macro_rules! impl_taiko_button {
  ($name:ident) => {
    impl Action<Client> for $name {
      fn execute(client: &mut Client, core: &mut Core<Client>, repeat: bool) -> bool {
        if repeat {
          return false;
        }

        match client.game_state {
          GameState::Playing => {
            client.gameplay_screen.hit(TaikoInput::$name, &core.graphics, &mut client.audio);

            return true;
          }

          _ => {}
        }

        return false;
      }
    }
  };
}

pub struct DonRight;
pub struct DonLeft;
pub struct KatLeft;
pub struct KatRight;

impl_taiko_button!(DonRight);
impl_taiko_button!(DonLeft);
impl_taiko_button!(KatLeft);
impl_taiko_button!(KatRight);
