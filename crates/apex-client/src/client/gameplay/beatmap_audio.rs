use std::ops::{Deref, DerefMut};

use rodio::Source;

use crate::core::{
  audio::{audio_engine::AudioEngine, audio_mixer::AudioController},
  time::{
    clock::{AbstractClock, Clock},
    time::Time,
  },
};

pub struct BeatmapAudio {
  audio: AudioController,
  clock: Clock,

  /// Delay before the first hit object.
  pub lead_in: Time,

  /// Delay after the last hit object.
  pub lead_out: Time,

  /// Offset of the audio.
  pub audio_offset: Time,
}

impl BeatmapAudio {
  pub fn new(audio: AudioController) -> Self {
    let clock = Clock::new();

    return Self {
      audio,
      clock,
      lead_in: Time::zero(),
      lead_out: Time::zero(),
      audio_offset: Time::zero(),
    };
  }

  pub fn borrow<'a>(&'a mut self, audio_engine: &'a mut AudioEngine) -> BorrowedBeatmapAudio<'a> {
    return BorrowedBeatmapAudio { beatmap_audio: self, audio_engine };
  }

  pub fn set_source<S>(&mut self, source: S)
  where
    S: Source<Item = f32> + Send + Sync + 'static,
  {
    self.audio.set_source(Box::new(source));
  }
}

pub struct BorrowedBeatmapAudio<'a> {
  beatmap_audio: &'a mut BeatmapAudio,
  audio_engine: &'a mut AudioEngine,
}

impl Deref for BorrowedBeatmapAudio<'_> {
  type Target = BeatmapAudio;

  fn deref(&self) -> &Self::Target {
    return self.beatmap_audio;
  }
}

impl DerefMut for BorrowedBeatmapAudio<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    return self.beatmap_audio;
  }
}

impl<'a> AbstractClock for BorrowedBeatmapAudio<'a> {
  fn is_playing(&self) -> bool {
    return self.beatmap_audio.clock.is_playing();
  }

  fn set_playing(&mut self, playing: bool) {
    self.beatmap_audio.clock.set_playing(playing);

    if !playing {
      self.audio_engine.pause();
    }
  }

  fn toggle(&mut self) {
    if self.beatmap_audio.clock.is_playing() {
      self.beatmap_audio.clock.set_playing(true);
    } else {
      self.audio_engine.pause();
      self.beatmap_audio.clock.set_playing(false);
    }
  }

  fn position(&mut self) -> Time {
    if self.audio_engine.is_paused() && self.is_playing() {
      self.audio_engine.play();
    }

    return self.beatmap_audio.clock.position() - self.beatmap_audio.lead_in + self.beatmap_audio.audio_offset;
  }

  fn set_position(&mut self, position: Time) {
    self.beatmap_audio.clock.set_position(position);
    _ = self.audio_engine.try_seek(position.into());
  }

  fn length(&self) -> Time {
    return self.beatmap_audio.clock.length();
  }

  fn set_length(&mut self, value: Time) {
    self.beatmap_audio.clock.set_length(value);
  }
}
