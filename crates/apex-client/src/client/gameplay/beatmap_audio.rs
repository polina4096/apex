use std::ops::{Deref, DerefMut};

use rodio::Source;

use crate::core::{
  audio::{audio_engine::AudioEngine, audio_mixer::AudioController},
  time::{clock::AbstractClock, time::Time},
};

pub struct BeatmapAudio {
  audio: AudioController,

  /// Delay before the first hit object.
  pub lead_in: Time,

  /// Delay after the last hit object.
  pub lead_out: Time,

  /// Offset of the audio.
  pub audio_offset: Time,
}

impl BeatmapAudio {
  pub fn new(audio: AudioController) -> Self {
    return Self {
      audio,
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
    self.audio.play_audio(source);
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
    return self.audio_engine.is_playing();
  }

  fn set_playing(&mut self, playing: bool) {
    self.audio_engine.set_playing(playing);
  }

  fn toggle(&mut self) {
    self.audio_engine.toggle();
  }

  fn position(&mut self) -> Time {
    let pos = self.audio_engine.position();

    return pos - self.beatmap_audio.lead_in + self.beatmap_audio.audio_offset;
  }

  fn set_position(&mut self, position: Time) {
    self.audio_engine.set_position(position + self.lead_in);
  }

  fn length(&self) -> Time {
    return self.audio_engine.length();
  }

  fn set_length(&mut self, value: Time) {
    self.audio_engine.set_length(value);
  }

  fn set_clock_position(&mut self, time: Time) {
    self.audio_engine.set_clock_position(time);
  }

  fn set_source_position(&mut self, time: Time) {
    self.audio_engine.set_source_position(time);
  }
}
