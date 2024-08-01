use crate::{
  client::client::Client,
  core::{core::Core, input::action::Action},
};

pub struct Debug;

impl Action<Client> for Debug {
  fn execute(client: &mut Client, _core: &mut Core<Client>, _repeat: bool) -> bool {
    client.debug_screen.toggle();

    return true;
  }
}
