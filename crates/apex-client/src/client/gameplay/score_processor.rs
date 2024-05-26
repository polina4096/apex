use crate::client::ui::ingame_overlay::ingame_overlay_view::HitResult;

pub struct ScoreProcessor {
  events: Vec<ScoreProcessorEvent>,

  result_300: usize,
  result_100: usize,
  result_miss: usize,
  accuracy: f32,
}

impl Default for ScoreProcessor {
  fn default() -> Self {
    return Self {
      events: Vec::new(),
      result_300: 0,
      result_100: 0,
      result_miss: 0,
      accuracy: 1.0,
    };
  }
}

impl ScoreProcessor {
  pub fn feed(&mut self, event: ScoreProcessorEvent) {
    match event.result {
      HitResult::Hit300 => self.result_300  += 1,
      HitResult::Hit100 => self.result_100  += 1,
      HitResult::Miss   => self.result_miss += 1,
    }

    self.accuracy = self.calc_accuracy();
    self.events.push(event);
  }

  pub fn accuracy(&self) -> f32 {
    return self.accuracy;
  }

  pub fn result_300s(&self) -> usize {
    return self.result_300;
  }

  pub fn result_100s(&self) -> usize {
    return self.result_100;
  }

  pub fn result_misses(&self) -> usize {
    return self.result_miss;
  }

  fn calc_accuracy(&self) -> f32 {
    let n_300 = self.result_300 as f32;
    let n_100 = self.result_100 as f32;
    let n_miss = self.result_miss as f32;

    return (n_300 + n_100 * 0.5) / (n_300 + n_100 + n_miss);
  }
}

pub struct ScoreProcessorEvent {
  pub result: HitResult,
}
