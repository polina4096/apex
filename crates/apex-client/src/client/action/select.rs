use apex_framework::{core::Core, input::action::Action};

use crate::client::client::{Client, GameState};

pub struct Select;

impl Action<Client> for Select {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
    match client.game_state {
      GameState::Selection => {
        client
          .selection_screen
          .beatmap_selector()
          .pick(&client.event_bus, &client.beatmap_cache)
          .unwrap_or_else(|err| {
            log::error!("Failed to select beatmap: {:?}", err);
          });

        return true;
      }

      GameState::Paused => {
        client.pause_screen.click();
      }

      _ => {}
    }

    return false;
  }
}
