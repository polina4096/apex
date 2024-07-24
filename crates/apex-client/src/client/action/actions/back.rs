use crate::{
  client::{
    action::action::Action,
    client::{Client, GameState},
  },
  core::{
    core::Core,
    time::{clock::AbstractClock, time::Time},
  },
};

pub struct Back;

impl Action<Client> for Back {
  fn execute(client: &mut Client, core: &mut Core<Client>, _repeat: bool) -> bool {
    match client.game_state {
      GameState::Selection => {
        if client.settings_screen.is_open() {
          client.settings_screen.toggle();
        } else if client.selection_screen.beatmap_selector().has_query() {
          client.selection_screen.beatmap_selector_mut().clear_query();
        } else {
          core.exit();
        }
      }

      GameState::Playing => {
        if client.settings_screen.is_open() {
          client.settings_screen.toggle();
        } else {
          let lead_in = Time::from_ms(client.settings.gameplay.lead_in() as f64);
          let delay_adjusted_position = client.audio_engine.position() - lead_in;
          let delay_adjusted_position = delay_adjusted_position.max(Time::zero());

          let selected = client.selection_screen.beatmap_selector().selected();
          if let Some((path, beatmap)) = client.beatmap_cache.get_index(selected) {
            Client::play_beatmap_audio_unchecked(&mut client.audio_engine, &mut client.audio_controller, path, beatmap);
            client.audio_engine.set_position(delay_adjusted_position);
          };

          client.game_state = GameState::Selection;
        }
      }

      GameState::Results => {
        client.game_state = GameState::Selection;
      }
    }

    return true;
  }
}
