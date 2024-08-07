use apex_framework::{core::Core, input::action::Action};

use crate::client::client::Client;

pub struct Settings;

impl Action<Client> for Settings {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
    client.settings_screen.toggle();

    return true;
  }
}
