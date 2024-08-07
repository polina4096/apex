use apex_framework::{core::Core, input::action::Action};

use crate::client::client::Client;

pub struct Debug;

impl Action<Client> for Debug {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
    client.debug_screen.toggle();

    return true;
  }
}
