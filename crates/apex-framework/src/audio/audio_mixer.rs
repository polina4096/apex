use std::{
  collections::VecDeque,
  sync::{
    mpsc::{Receiver, Sender},
    Arc,
  },
};

use parking_lot::Mutex;
use rodio::{Sample as _, Source};

pub enum AudioMixerEvent {
  PlaySound(Box<dyn Source<Item = f32> + Send + Sync>),
  SetMasterVolume(f32),
  SetAudioVolume(f32),
  SetSoundVolume(f32),
}

pub struct AudioMixer {
  source: Arc<Mutex<Box<dyn Source<Item = f32> + Send + Sync>>>,
  sounds: VecDeque<Box<dyn Source<Item = f32> + Send + Sync>>,
  rx: Receiver<AudioMixerEvent>,

  master_volume: f32,
  music_volume: f32,
  effect_volume: f32,
}

pub fn mixer(
  source: impl Source<Item = f32> + Send + Sync + 'static,
  master_volume: f32,
  audio_volume: f32,
  sound_volume: f32,
) -> (AudioMixer, AudioController) {
  let (tx, rx) = std::sync::mpsc::channel();
  let source = Arc::new(Mutex::new(Box::new(source) as Box<dyn Source<Item = f32> + Send + Sync>));
  let sounds = VecDeque::new();

  let mixer = AudioMixer {
    source: source.clone(),
    sounds,
    rx,

    master_volume,
    music_volume: audio_volume,
    effect_volume: sound_volume,
  };

  let controller = AudioController { tx, source };

  return (mixer, controller);
}

impl Iterator for AudioMixer {
  type Item = f32;

  fn next(&mut self) -> Option<Self::Item> {
    if let Ok(event) = self.rx.try_recv() {
      match event {
        AudioMixerEvent::PlaySound(sound) => {
          self.sounds.push_back(sound);
        }

        AudioMixerEvent::SetMasterVolume(volume) => self.master_volume = volume,
        AudioMixerEvent::SetAudioVolume(volume) => self.music_volume = volume,
        AudioMixerEvent::SetSoundVolume(volume) => self.effect_volume = volume,
      }
    }

    let mut sample = f32::zero_value();

    self.sounds.retain_mut(|sound| {
      if let Some(sound_sample) = sound.next() {
        sample = sample.saturating_add(sound_sample.amplify(self.effect_volume));
        return true;
      }

      return false;
    });

    {
      let mut lock = self.source.lock();
      if let Some(source_sample) = lock.next() {
        sample = source_sample.amplify(self.music_volume).saturating_add(sample).amplify(self.master_volume);
      }
    }

    return Some(sample);
  }
}

impl Source for AudioMixer {
  fn current_frame_len(&self) -> Option<usize> {
    return None;
  }

  fn channels(&self) -> u16 {
    return self.source.lock().channels();
  }

  fn sample_rate(&self) -> u32 {
    return self.source.lock().sample_rate();
  }

  fn total_duration(&self) -> Option<instant::Duration> {
    return None;
  }

  fn try_seek(&mut self, pos: instant::Duration) -> Result<(), rodio::source::SeekError> {
    return self.source.lock().try_seek(pos);
  }
}

#[derive(Clone)]
pub struct AudioController {
  source: Arc<Mutex<Box<dyn Source<Item = f32> + Send + Sync>>>,
  tx: Sender<AudioMixerEvent>,
}

impl AudioController {
  pub fn play_sound(&self, sound: impl Source<Item = f32> + Send + Sync + 'static) {
    self.tx.send(AudioMixerEvent::PlaySound(Box::new(sound))).unwrap();
  }

  pub fn play_audio(&self, source: impl Source<Item = f32> + Send + Sync + 'static) {
    *self.source.lock() = Box::new(source);
  }

  pub fn set_master_volume(&self, volume: f32) {
    self.tx.send(AudioMixerEvent::SetMasterVolume(volume)).unwrap();
  }

  pub fn set_audio_volume(&self, volume: f32) {
    self.tx.send(AudioMixerEvent::SetAudioVolume(volume)).unwrap();
  }

  pub fn set_sound_volume(&self, volume: f32) {
    self.tx.send(AudioMixerEvent::SetSoundVolume(volume)).unwrap();
  }
}
