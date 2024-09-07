use std::{fs::File, path::Path};

use apex_framework::{
  audio::{
    arc_buffer::ArcSamplesBuffer, audio_engine::AudioEngine, audio_mixer::AudioController,
    frameless_source::FramelessSource, lead_in::lead_in,
  },
  time::{clock::AbstractClock, time::Time},
};
use rodio::{source::UniformSourceIterator, Decoder, Device, DeviceTrait as _, Source};

/// Audio wrapper which allows for leading and trailing additional delays, or other gameplay-specific things.
pub struct GameAudio {
  audio_engine: AudioEngine,
  audio_controller: AudioController,
  config: rodio::SupportedStreamConfig,

  /// Delay before the first hit object.
  pub lead_in: Time,

  /// Delay after the last hit object.
  pub lead_out: Time,

  /// Offset of the audio.
  pub audio_offset: Time,
}

impl GameAudio {
  pub fn new(audio_engine: AudioEngine, audio_controller: AudioController) -> Self {
    let config = audio_engine.device().default_output_config().unwrap();

    return Self {
      audio_engine,
      audio_controller,
      config,
      lead_in: Time::zero(),
      lead_out: Time::zero(),
      audio_offset: Time::zero(),
    };
  }

  pub fn with_lead_in(mut self, lead_in: Time) -> Self {
    self.lead_in = lead_in;
    return self;
  }

  pub fn with_lead_out(mut self, lead_out: Time) -> Self {
    self.lead_out = lead_out;
    return self;
  }

  pub fn set_source<S>(&mut self, source: S)
  where
    S: Source<Item = f32> + Send + Sync + 'static,
  {
    self
      .audio_controller
      .play_audio(lead_in(source, std::time::Duration::from_millis(self.lead_in.to_ms() as u64)));
  }

  pub fn device(&self) -> &Device {
    return self.audio_engine.device();
  }

  pub fn controller(&self) -> GameAudioController {
    return GameAudioController(self.audio_controller.clone());
  }

  pub fn play_sound(&self, sound: impl Source<Item = f32> + Send + Sync + 'static) {
    self.audio_controller.play_sound(sound);
  }

  pub fn set_master_volume(&self, volume: f32) {
    self.audio_controller.set_master_volume(volume);
  }

  pub fn set_music_volume(&self, volume: f32) {
    self.audio_controller.set_audio_volume(volume);
  }

  pub fn set_effect_volume(&self, volume: f32) {
    self.audio_controller.set_sound_volume(volume);
  }

  pub fn load_sound(&self, path: impl AsRef<Path>) -> ArcSamplesBuffer<f32> {
    let channels = self.config.channels();
    let sample_rate = self.config.sample_rate();
    let source = Decoder::new(File::open(path).unwrap()).unwrap();

    // FramelessSource is needed for a audio desync workaround, see https://github.com/RustAudio/rodio/issues/316
    let source = UniformSourceIterator::new(FramelessSource::new(source), channels, sample_rate.0);

    return ArcSamplesBuffer::<f32>::new(channels, sample_rate.0, source.collect::<Vec<_>>());
  }
}

impl AbstractClock for GameAudio {
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

    return pos - self.lead_in + self.audio_offset;
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
}

pub struct GameAudioController(AudioController);

impl GameAudioController {
  pub fn play_sound(&self, sound: impl Source<Item = f32> + Send + Sync + 'static) {
    self.0.play_sound(sound);
  }

  pub fn set_master_volume(&self, volume: f32) {
    self.0.set_master_volume(volume);
  }

  pub fn set_music_volume(&self, volume: f32) {
    self.0.set_audio_volume(volume);
  }

  pub fn set_effect_volume(&self, volume: f32) {
    self.0.set_sound_volume(volume);
  }
}
