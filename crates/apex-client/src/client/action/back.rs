use crate::{
  client::client::{Client, GameState},
  core::{core::Core, input::action::Action, time::clock::AbstractClock},
};

pub struct Back;

impl Action<Client> for Back {
  fn execute(client: &mut Client, core: &mut Core<Client>, repeat: bool) -> bool {
    if repeat {
      return false;
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
