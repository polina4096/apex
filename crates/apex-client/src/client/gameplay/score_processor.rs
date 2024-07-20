use crate::client::ui::ingame_overlay::HitResult;

pub struct ScoreProcessor {
  events: Vec<ScoreProcessorEvent>,

  result_300: usize,
  result_150: usize,
  result_miss: usize,
  curr_combo: usize,
  max_combo: usize,
  accuracy: f32,
}

impl Default for ScoreProcessor {
  fn default() -> Self {
    return Self {
      events: Vec::new(),
      result_300: 0,
      result_150: 0,
      result_miss: 0,
      curr_combo: 0,
      max_combo: 0,
      accuracy: 1.0,
    };
  }
}

impl ScoreProcessor {
  pub fn feed(&mut self, event: ScoreProcessorEvent) {
    match event.result {
      HitResult::Hit300 => {
        self.result_300 += 1;
        self.curr_combo += 1;
      }

      HitResult::Hit150 => {
        self.result_150 += 1;
        self.curr_combo += 1;
      }

      HitResult::Miss => {
        self.result_miss += 1;
        self.curr_combo = 0;
      }
    };

    if self.curr_combo > self.max_combo {
      self.max_combo = self.curr_combo;
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

  pub fn result_150s(&self) -> usize {
    return self.result_150;
  }

  pub fn result_misses(&self) -> usize {
    return self.result_miss;
  }

  pub fn curr_combo(&self) -> usize {
    return self.curr_combo;
  }

  pub fn max_combo(&self) -> usize {
    return self.max_combo;
  }

  fn calc_accuracy(&self) -> f32 {
    let n_300 = self.result_300 as f32;
    let n_150 = self.result_150 as f32;
    let n_miss = self.result_miss as f32;

    return (n_300 + n_150 * 0.5) / (n_300 + n_150 + n_miss);
  }
}

pub struct ScoreProcessorEvent {
  pub result: HitResult,
}
