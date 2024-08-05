use log::error;
use rodio::{Device, OutputStream, OutputStreamHandle, Sink, Source};

use crate::core::time::{
  clock::{AbstractClock, Clock},
  time::Time,
};

use super::OutputStreamExt as _;

pub struct AudioEngine {
  #[allow(unused)]
  stream: OutputStream,

  #[allow(unused)]
  stream_handle: OutputStreamHandle,

  device: Device,
  sink: Sink,
  clock: Clock,
}

impl AudioEngine {
  pub fn new() -> Self {
    let (device, (stream, stream_handle)) = OutputStream::try_default_device().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let clock = Clock::new();

    return Self {
      stream,
      stream_handle,
      device,
      sink,
      clock,
    };
  }

  pub fn with_source<S>(source: S) -> Self
  where
    S: Source<Item = f32> + Send + 'static,
  {
    let mut engine = Self::new();
    engine.set_source(source);

    return engine;
  }

  pub fn set_source<S>(&mut self, source: S)
  where
    S: Source<Item = f32> + Send + 'static,
  {
    self.sink.clear();
    self.sink.append(source);
  }

  pub fn clear_source(&mut self) {
    self.sink.clear();
  }

  pub fn device(&self) -> &Device {
    return &self.device;
  }
}

impl AbstractClock for AudioEngine {
  fn is_playing(&self) -> bool {
    return self.clock.is_playing();
  }

  fn set_playing(&mut self, playing: bool) {
    self.clock.set_playing(playing);

    if !playing {
      self.sink.pause();
    } else {
      self.sink.play();
    }
  }

  fn toggle(&mut self) {
    if !self.clock.is_playing() {
      self.clock.set_playing(true);
      self.sink.play();
    } else {
      self.clock.set_playing(false);
      self.sink.pause();
    }
  }

  fn position(&mut self) -> Time {
    return self.clock.position();
  }

  fn set_position(&mut self, position: Time) {
    self.clock.set_position(position);
    if let Err(e) = self.sink.try_seek(position.into()) {
      error!("Failed to seek audio source: {:?}", e);
    }
  }

  fn length(&self) -> Time {
    return self.clock.length();
  }

  fn set_length(&mut self, value: Time) {
    self.clock.set_length(value);
  }
}
