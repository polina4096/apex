use apex_framework::{core::Core, input::action::Action};

use crate::client::client::{Client, GameState};

pub struct Next;

impl Action<Client> for Next {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
    match client.game_state {
      GameState::Selection => {
        client.selection_screen.beatmap_selector_mut().select_next();
        client.play_beatmap_audio();

        return true;
      }

      GameState::Paused => {
        client.pause_screen.select_down();
      }

      _ => {}
    }

    return false;
  }
}
