use apex_framework::{core::Core, input::action::Action};

use crate::client::client::Client;

pub struct Settings;

impl Action<Client> for Settings {
  fn execute(client: &mut Client, _core: &mut Core<Client>, repeat: bool) -> bool {
    if repeat {
      return false;
    }

    client.settings_screen.toggle();

    return true;
  }
}
