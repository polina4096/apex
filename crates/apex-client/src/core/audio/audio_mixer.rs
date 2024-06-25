use std::{
  collections::VecDeque,
  sync::mpsc::{Receiver, Sender},
};

use rodio::{dynamic_mixer::DynamicMixer, Sample as _, Source};

pub enum AudioMixerEvent {
  AddOneshot(Box<dyn Source<Item = f32> + Send + Sync>),
  SetSource(Box<dyn Source<Item = f32> + Send + Sync>),
}

pub struct AudioMixer {
  source: Box<dyn Source<Item = f32> + Send + Sync>,
  sounds: VecDeque<Box<dyn Source<Item = f32> + Send + Sync>>,
  rx: Receiver<AudioMixerEvent>,
}

pub fn mixer(source: impl Source<Item = f32> + Send + Sync + 'static) -> (AudioMixer, AudioController) {
  let (tx, rx) = std::sync::mpsc::channel();
  let source = Box::new(source);
  let sounds = VecDeque::new();

  let mixer = AudioMixer { source, sounds, rx };
  let controller = AudioController { tx };

  return (mixer, controller);
}

impl Iterator for AudioMixer {
  type Item = f32;

  fn next(&mut self) -> Option<Self::Item> {
    if let Ok(event) = self.rx.try_recv() {
      match event {
        AudioMixerEvent::AddOneshot(sound) => {
          self.sounds.push_back(sound);
        }

        AudioMixerEvent::SetSource(source) => {
          self.source = source;
        }
      }
    }

    let mut sample = f32::zero_value();

    self.sounds.retain_mut(|sound| {
      if let Some(sound_sample) = sound.next() {
        sample = sample.saturating_add(sound_sample);
        return true;
      }

      return false;
    });

    if let Some(source_sample) = self.source.next() {
      sample = source_sample.saturating_add(sample);
    }

    return Some(sample);
  }
}

impl Source for AudioMixer {
  fn current_frame_len(&self) -> Option<usize> {
    return None;
  }

  fn channels(&self) -> u16 {
    return self.source.channels();
  }

  fn sample_rate(&self) -> u32 {
    return self.source.sample_rate();
  }

  fn total_duration(&self) -> Option<instant::Duration> {
    return None;
  }

  fn try_seek(&mut self, pos: instant::Duration) -> Result<(), rodio::source::SeekError> {
    return self.source.try_seek(pos);
  }
}

#[derive(Clone)]
pub struct AudioController {
  tx: Sender<AudioMixerEvent>,
}

impl AudioController {
  pub fn play_sound(&mut self, sound: Box<dyn Source<Item = f32> + Send + Sync>) {
    self.tx.send(AudioMixerEvent::AddOneshot(sound)).unwrap();
  }

  pub fn set_source(&mut self, source: Box<dyn Source<Item = f32> + Send + Sync>) {
    self.tx.send(AudioMixerEvent::SetSource(source)).unwrap();
  }
}
