use crate::{
  client::client::{Client, GameState},
  core::{core::Core, input::action::Action},
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

      GameState::Paused => {
        client.pause_screen.select_up();
      }

      _ => {}
    }

    return false;
  }
}
