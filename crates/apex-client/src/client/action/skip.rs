use crate::{
  client::client::{Client, GameState},
  core::{core::Core, input::action::Action, time::time::Time},
};

pub struct Skip;

impl Action<Client> for Skip {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
    match client.game_state {
      GameState::Playing => {
        client.gameplay_screen.skip_break(&mut client.audio_engine, Time::from_seconds(1.0));

        return true;
      }

      _ => {}
    }

    return false;
  }
}
