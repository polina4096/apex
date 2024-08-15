use apex_framework::{core::Core, input::action::Action};

use crate::client::{
  client::{Client, GameState},
  event::ClientEvent,
};

pub struct Retry;

impl Action<Client> for Retry {
  fn execute(client: &mut Client, _core: &mut Core<Client>, repeat: bool) -> bool {
    if repeat {
      return false;
    }

    match client.game_state {
      GameState::Playing => {
        client.event_bus.send(ClientEvent::RetryBeatmap);

        return true;
      }

      _ => {}
    }

    return false;
  }
}
