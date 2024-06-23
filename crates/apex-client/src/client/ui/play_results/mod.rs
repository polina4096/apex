use crate::{
  client::{client::Client, state::AppState},
  core::core::Core,
};

pub struct PlayResultsView {}

impl PlayResultsView {
  pub fn prepare(&mut self, core: &Core<Client>, state: &mut AppState) {}
}
