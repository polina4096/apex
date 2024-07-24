use crate::{
  client::{
    action::action::Action,
    client::{Client, GameState},
  },
  core::core::Core,
};

pub struct Prev;

impl Action<Client> for Prev {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
    match client.game_state {
      GameState::Selection => {
        client.selection_screen.beatmap_selector_mut().select_prev();
        client.play_beatmap_audio();

        return true;
      }

      _ => {}
    }

    return false;
  }
}
