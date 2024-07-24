use crate::{
  client::{action::action::Action, client::Client},
  core::core::Core,
};

pub struct Recording;

impl Action<Client> for Recording {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
    client.recording_screen.toggle();

    return true;
  }
}
