use rodio::{cpal::FromSample, OutputStream, OutputStreamHandle, Sample, Sink, Source};

use crate::core::time::{
  clock::{AbstractClock, Clock},
  time::Time,
};

pub struct BeatmapAudio {
  #[allow(unused)]
  stream: OutputStream,

  #[allow(unused)]
  stream_handle: OutputStreamHandle,

  sink: Sink,
  clock: Clock,

  /// Delay before the first hit object.
  pub lead_in: Time,

  /// Delay after the last hit object.
  pub lead_out: Time,

  /// Offset of the audio.
  pub audio_offset: Time,
}

impl BeatmapAudio {
  pub fn new() -> Self {
    let clock = Clock::new();

    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    return Self {
      stream,
      stream_handle,
      sink,
      clock,
      lead_in: Time::zero(),
      lead_out: Time::zero(),
      audio_offset: Time::zero(),
    };
  }

  pub fn set_source<S>(&mut self, source: S)
  where
    S: Source + Send + 'static,
    f32: FromSample<S::Item>,
    S::Item: Sample + Send,
  {
    self.sink.clear();
    self.sink.append(source);
  }
}

impl AbstractClock for BeatmapAudio {
  fn is_playing(&self) -> bool {
    return self.clock.is_playing();
  }

  fn set_playing(&mut self, playing: bool) {
    self.clock.set_playing(playing);

    if !playing {
      self.sink.pause();
    }
  }

  fn toggle(&mut self) {
    if self.clock.is_playing() {
      self.clock.set_playing(true);
    } else {
      self.sink.pause();
      self.clock.set_playing(false);
    }
  }

  fn position(&mut self) -> Time {
    if self.sink.is_paused() && self.is_playing() && self.clock.position() > self.lead_in {
      self.sink.play();
    }

    return self.clock.position() - self.lead_in + self.audio_offset;
  }

  fn set_position(&mut self, position: Time) {
    self.clock.set_position(position);
    self.sink.try_seek(position.into()).unwrap();
  }

  fn length(&self) -> Time {
    return self.clock.length();
  }

  fn set_length(&mut self, value: Time) {
    self.clock.set_length(value);
  }
}
