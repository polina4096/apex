use crate::{
  client::{
    action::action::Action,
    client::{Client, GameState},
    event::ClientEvent,
  },
  core::core::Core,
};

pub struct Retry;

impl Action<Client> for Retry {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
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
