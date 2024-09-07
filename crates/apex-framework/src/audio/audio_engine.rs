use log::error;
use rodio::{Device, OutputStream, OutputStreamHandle, Sink, Source};
use thiserror::Error;

use crate::time::{
  clock::{AbstractClock, Clock},
  time::Time,
};

#[derive(Debug, Error)]
pub enum AudioEngineError {
  #[error("Failed to acquire output stream")]
  StreamError(#[from] rodio::StreamError),

  #[error("Failed to create audio sink")]
  PlayError(#[from] rodio::PlayError),
}

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
  pub fn try_new(device: rodio::Device) -> Result<Self, AudioEngineError> {
    let (stream, stream_handle) = OutputStream::try_from_device(&device)?;
    let sink = Sink::try_new(&stream_handle)?;
    let clock = Clock::new();

    return Ok(Self {
      stream,
      stream_handle,
      device,
      sink,
      clock,
    });
  }

  pub fn try_with_source<S>(device: rodio::Device, source: S) -> Result<Self, AudioEngineError>
  where
    S: Source<Item = f32> + Send + 'static,
  {
    let mut engine = Self::try_new(device)?;

    engine.set_source(source);

    return Ok(engine);
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
