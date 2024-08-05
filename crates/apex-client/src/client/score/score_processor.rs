use jiff::Timestamp;

use crate::{client::ui::ingame_overlay::HitResult, core::time::time::Time};

use super::{grades::Grade, score::Score};

pub struct ScoreProcessor {
  score_points: usize,
  result_300: usize,
  result_150: usize,
  result_miss: usize,
  curr_combo: usize,
  max_combo: usize,
  accuracy: f32,

  hits: Vec<Time>,
}

impl Default for ScoreProcessor {
  fn default() -> Self {
    return Self {
      hits: Vec::new(),
      score_points: 0,
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
  pub fn feed(&mut self, time: Time, result: HitResult) {
    match result {
      HitResult::Hit300 => {
        self.result_300 += 1;
        self.curr_combo += 1;

        self.score_points += 300 * self.curr_combo;
      }

      HitResult::Hit150 => {
        self.result_150 += 1;
        self.curr_combo += 1;

        self.score_points += 150 * self.curr_combo;
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
    self.hits.push(time);
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

  pub fn export(&self, date: Timestamp, username: String) -> Score {
    return Score {
      date,
      username,
      score_points: self.score_points,
      result_300: self.result_300,
      result_150: self.result_150,
      result_miss: self.result_miss,
      last_combo: self.curr_combo,
      max_combo: self.max_combo,
      accuracy: self.accuracy,
      grade: Grade::from_osu_stable(self.result_300, self.result_150, self.result_miss),
      hits: self.hits.clone(),
    };
  }
}
