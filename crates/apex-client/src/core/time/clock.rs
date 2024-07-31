use instant::Instant;

use super::time::Time;

pub trait AbstractClock {
  fn is_playing(&self) -> bool;
  fn set_playing(&mut self, value: bool);
  fn toggle(&mut self);

  fn position(&mut self) -> Time;
  fn set_position(&mut self, time: Time);
  fn set_clock_position(&mut self, time: Time);
  fn set_source_position(&mut self, time: Time);

  fn length(&self) -> Time;
  fn set_length(&mut self, value: Time);
}

pub struct Clock {
  last_pause: Instant,
  last_time: Time,

  playing: bool,

  length: Time,
}

impl Clock {
  pub fn new() -> Self {
    return Self {
      last_pause: Instant::now(),
      last_time: Time::zero(),

      playing: false,

      length: Time::zero(),
    };
  }
}

impl AbstractClock for Clock {
  fn set_position(&mut self, value: Time) {
    self.last_pause = Instant::now();
    self.last_time = value;
  }

  fn position(&mut self) -> Time {
    if self.playing {
      let now = instant::Instant::now();
      let diff = now.duration_since(self.last_pause);
      let time = self.last_time + diff.into();

      return time;
    } else {
      return self.last_time;
    }
  }

  fn is_playing(&self) -> bool {
    return self.playing;
  }

  fn set_playing(&mut self, playing: bool) {
    let now = instant::Instant::now();
    let diff = now.duration_since(self.last_pause);
    let time = self.last_time + diff.into();

    if playing {
      self.last_pause = now;
    } else if self.playing {
      self.last_time = time;
    }

    self.playing = playing;
  }

  fn toggle(&mut self) {
    self.set_playing(!self.playing);
  }

  fn set_length(&mut self, value: Time) {
    self.length = value;
  }

  fn length(&self) -> Time {
    return self.length;
  }

  fn set_clock_position(&mut self, _time: Time) {
    unimplemented!()
  }

  fn set_source_position(&mut self, _time: Time) {
    unimplemented!()
  }
}
