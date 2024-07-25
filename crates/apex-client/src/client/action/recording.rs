use crate::{
  client::client::Client,
  core::{core::Core, input::action::Action},
};

pub struct Recording;

impl Action<Client> for Recording {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
    client.recording_screen.toggle();

    return true;
  }
}
