use std::{
  cell::Cell,
  collections::VecDeque,
  sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
  },
};

use rodio::{Sample as _, Source};

pub enum AudioMixerEvent {
  SetSource(Box<dyn Source<Item = f32> + Send + Sync>),
}

pub struct AudioMixer {
  source: Arc<Mutex<Box<dyn Source<Item = f32> + Send + Sync>>>,
  sounds: VecDeque<Box<dyn Source<Item = f32> + Send + Sync>>,
  rx: Receiver<Box<dyn Source<Item = f32> + Send + Sync>>,
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

  let mixer = AudioMixer { source: source.clone(), sounds, rx };

  let controller = AudioController {
    tx,
    source,
    master_volume: Cell::new(master_volume),
    audio_volume: Cell::new(audio_volume),
    sound_volume: Cell::new(sound_volume),
  };

  return (mixer, controller);
}

impl Iterator for AudioMixer {
  type Item = f32;

  fn next(&mut self) -> Option<Self::Item> {
    if let Ok(sound) = self.rx.try_recv() {
      self.sounds.push_back(sound);
    }

    let mut sample = f32::zero_value();

    self.sounds.retain_mut(|sound| {
      if let Some(sound_sample) = sound.next() {
        sample = sample.saturating_add(sound_sample);
        return true;
      }

      return false;
    });

    {
      let mut lock = self.source.lock().unwrap();
      if let Some(source_sample) = lock.next() {
        sample = source_sample.saturating_add(sample);
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
    return self.source.lock().unwrap().channels();
  }

  fn sample_rate(&self) -> u32 {
    return self.source.lock().unwrap().sample_rate();
  }

  fn total_duration(&self) -> Option<instant::Duration> {
    return None;
  }

  fn try_seek(&mut self, pos: instant::Duration) -> Result<(), rodio::source::SeekError> {
    return self.source.lock().unwrap().try_seek(pos);
  }
}

#[derive(Clone)]
pub struct AudioController {
  source: Arc<Mutex<Box<dyn Source<Item = f32> + Send + Sync>>>,
  tx: Sender<Box<dyn Source<Item = f32> + Send + Sync>>,

  master_volume: Cell<f32>,
  audio_volume: Cell<f32>,
  sound_volume: Cell<f32>,
}

impl AudioController {
  pub fn play_sound(&mut self, sound: impl Source<Item = f32> + Send + Sync + 'static) {
    self.tx.send(Box::new(sound)).unwrap();
  }

  pub fn play_audio(&mut self, source: impl Source<Item = f32> + Send + Sync + 'static) {
    *self.source.lock().unwrap() = Box::new(source);
  }

  pub fn set_master_volume(&self, volume: f32) {
    self.master_volume.set(volume);
  }

  pub fn set_audio_volume(&self, volume: f32) {
    self.audio_volume.set(volume);
  }

  pub fn set_sound_volume(&self, volume: f32) {
    self.sound_volume.set(volume);
  }
}
