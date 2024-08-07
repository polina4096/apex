use apex_framework::time::time::Time;
use jiff::Timestamp;

use crate::client::gameplay::taiko_player::TaikoInput;

use super::grades::Grade;

#[derive(Debug)]
pub struct Score {
  pub(crate) date: Timestamp,
  pub(crate) username: String,
  pub(crate) score_points: usize,
  pub(crate) result_300: usize,
  pub(crate) result_150: usize,
  pub(crate) result_miss: usize,
  pub(crate) last_combo: usize,
  pub(crate) max_combo: usize,
  pub(crate) accuracy: f32,
  pub(crate) grade: Grade,
  pub(crate) hits: Vec<(Time, TaikoInput)>,
}

impl Default for Score {
  fn default() -> Self {
    Self {
      date: Timestamp::default(),
      username: String::from(Score::DEFAULT_USERNAME),
      score_points: 0,
      result_300: 0,
      result_150: 0,
      result_miss: 0,
      last_combo: 0,
      max_combo: 0,
      accuracy: 0.0,
      grade: Grade::D,
      hits: Vec::new(),
    }
  }
}

impl Score {
  pub const DEFAULT_USERNAME: &'static str = "player";

  pub fn date(&self) -> Timestamp {
    return self.date;
  }

  pub fn username(&self) -> &str {
    return &self.username;
  }

  pub fn score_points(&self) -> usize {
    return self.score_points;
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

  pub fn last_combo(&self) -> usize {
    return self.last_combo;
  }

  pub fn max_combo(&self) -> usize {
    return self.max_combo;
  }

  pub fn accuracy(&self) -> f32 {
    return self.accuracy;
  }

  pub fn grade(&self) -> Grade {
    return self.grade;
  }

  pub fn hits(&self) -> &[(Time, TaikoInput)] {
    return &self.hits;
  }
}
