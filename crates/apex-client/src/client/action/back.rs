use apex_framework::{core::Core, input::action::Action, time::clock::AbstractClock as _};

use crate::client::client::{Client, GameState};

pub struct Back;

impl Action<Client> for Back {
  fn execute(client: &mut Client, core: &mut Core<Client>, repeat: bool) -> bool {
    if repeat {
      return false;
    }

    if client.settings_screen.is_open() {
      client.settings_screen.toggle();
      return true;
    }

    match client.game_state {
      GameState::Selection => {
        if client.selection_screen.beatmap_selector().has_query() {
          client.selection_screen.beatmap_selector_mut().clear_query();
          client.selection_screen.scroll_to_selected();
        } else {
          core.exit();
        }
      }

      GameState::Playing => {
        client.audio.set_playing(false);
        client.game_state = GameState::Paused;
      }

      GameState::Paused => {
        client.pause_screen.deselect();
        client.audio.set_playing(true);
        client.game_state = GameState::Playing;
      }

      GameState::Results => {
        client.game_state = GameState::Selection;
      }
    }

    return true;
  }
}
