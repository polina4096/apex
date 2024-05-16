use rodio::Sink;

use crate::{client::graphics::taiko_renderer::taiko_renderer::TaikoRenderer, core::{graphics::graphics::Graphics, time::{clock::{AbstractClock as _, Clock}, time::Time}}};

use super::playback_controller::PlaybackController;

pub struct GameplayPlaybackController<'a> {
  pub taiko_renderer: &'a mut TaikoRenderer,
  pub clock: &'a mut Clock,
  pub sink: &'a mut Sink,
}

impl PlaybackController for GameplayPlaybackController<'_> {
  fn set_playing(&mut self, playing: bool) {
    self.taiko_renderer.culling = 0;
    self.taiko_renderer.hit_idx = 0;

    self.clock.set_playing(playing);

    if playing {
      self.sink.play();
    } else {
      self.sink.pause();
    }
  }

  fn is_playing(&self) -> bool {
    return self.clock.is_playing();
  }

  fn toggle(&mut self) {
    if self.clock.is_playing() {
      self.sink.play();
      self.clock.set_playing(true);
      self.taiko_renderer.culling = 0;
      self.taiko_renderer.hit_idx = 0;
    } else {
      self.sink.pause();
      self.clock.set_playing(false);
      self.taiko_renderer.culling = 0;
      self.taiko_renderer.hit_idx = 0;
    }
  }

  fn set_position(&mut self, position: Time) {
    self.taiko_renderer.culling = 0;
    self.taiko_renderer.hit_idx = 0;
    self.clock.set_position(position);
    self.sink.try_seek(position.into()).unwrap();
  }

  fn position(&mut self) -> Time {
    return self.clock.position();
  }

  fn length(&self) -> Time {
    return self.clock.length();
  }
}
