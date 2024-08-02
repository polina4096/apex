use crate::{
  client::client::{Client, GameState},
  core::{core::Core, input::action::Action},
};

pub struct ClearQuery;

impl Action<Client> for ClearQuery {
  fn execute(client: &mut Client, _core: &mut Core<Client>, repeat: bool) -> bool {
    if repeat {
      return false;
    }

    match client.game_state {
      GameState::Selection => {
        client.selection_screen.beatmap_selector_mut().clear_query();
      }

      _ => {}
    }

    return true;
  }
}
