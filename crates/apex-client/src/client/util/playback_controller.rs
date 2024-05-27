use crate::core::time::time::Time;

pub trait PlaybackController {
  fn set_playing(&mut self, playing: bool);
  fn is_playing(&self) -> bool;
  fn toggle(&mut self);

  fn set_position(&mut self, position: Time);
  fn position(&mut self) -> Time;

  fn length(&self) -> Time;
}
