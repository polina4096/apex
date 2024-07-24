use crate::{
  client::{
    action::action::Action,
    client::{Client, GameState},
  },
  core::core::Core,
};

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

      _ => {}
    }

    return false;
  }
}
