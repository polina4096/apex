use crate::{
  client::{
    action::action::Action,
    client::{Client, GameState},
    gameplay::taiko_player::TaikoPlayerInput,
  },
  core::core::Core,
};

macro_rules! impl_taiko_button {
  ($name:ident) => {
    impl Action<Client> for $name {
      fn execute(client: &mut Client, core: &mut Core<Client>, repeat: bool) -> bool {
        if repeat {
          return true;
        }

        match client.game_state {
          GameState::Playing => {
            client.gameplay_screen.hit(TaikoPlayerInput::$name, &core.graphics, &mut client.audio_engine);

            return true;
          }

          _ => {}
        }

        return false;
      }
    }
  };
}

pub struct KatOne;
pub struct KatTwo;
pub struct DonOne;
pub struct DonTwo;

impl_taiko_button!(KatOne);
impl_taiko_button!(KatTwo);
impl_taiko_button!(DonOne);
impl_taiko_button!(DonTwo);
