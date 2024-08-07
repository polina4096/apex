use apex_framework::{core::Core, input::action::Action};

use crate::client::client::Client;

pub struct Recording;

impl Action<Client> for Recording {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
    client.recording_screen.toggle();

    return true;
  }
}
