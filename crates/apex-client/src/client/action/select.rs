use apex_framework::{core::Core, input::action::Action};

use crate::client::{
  client::{Client, GameState},
  event::ClientEvent,
};

pub struct Select;

impl Action<Client> for Select {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
    match client.game_state {
      GameState::Selection => {
        let selected_idx = client.selection_screen.beatmap_selector().selected();
        #[rustfmt::skip] let Some((path, _)) = client.beatmap_cache.get_index(selected_idx) else {
          log::error!("Failed to select beatmap, no beatmap with cache idx `{}` found.", selected_idx);
          return true;
         };

        client.event_bus.send(ClientEvent::PickBeatmap { path: path.clone() });

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
