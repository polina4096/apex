use apex_framework::time::{clock::AbstractClock as _, time::Time};

use crate::client::{
  audio::game_audio::GameAudio,
  score::judgement_processor::{check_hit, HitResult},
};

use super::beatmap::{calc_hit_window_150, calc_hit_window_300, Beatmap, BreakPoint};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaikoInput {
  DonRight = 0,
  DonLeft = 1,
  KatLeft = 2,
  KatRight = 3,
}

impl TryFrom<u8> for TaikoInput {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(TaikoInput::DonRight),
      1 => Ok(TaikoInput::DonLeft),
      2 => Ok(TaikoInput::KatLeft),
      3 => Ok(TaikoInput::KatRight),
      _ => Err(()),
    }
  }
}

#[derive(Debug, Clone)]
pub enum BreakState {
  /// No break is currently active.
  None,

  /// A break is currently active.
  Break(BreakPoint),

  /// Beatmap is currently in the skippable intro phase.
  Intro(BreakPoint),
}

/// Logcial actions that a player can perform while playing taiko.
pub struct TaikoPlayer {
  beatmap: Beatmap,

  // Memoized for performance reasons.
  hit_window_150: Time,
  hit_window_300: Time,

  current_circle: usize,
  current_break_point: usize,
}

impl TaikoPlayer {
  pub fn new() -> Self {
    return Self {
      beatmap: Beatmap::default(),
      hit_window_150: Time::zero(),
      hit_window_300: Time::zero(),
      current_circle: 0,
      current_break_point: 0,
    };
  }

  pub fn play(&mut self, beatmap: Beatmap) {
    self.reset();

    self.beatmap = beatmap;
    self.hit_window_150 = calc_hit_window_150(self.beatmap.overall_difficulty);
    self.hit_window_300 = calc_hit_window_300(self.beatmap.overall_difficulty);
  }

  pub fn reset(&mut self) {
    self.current_circle = 0;
    self.current_break_point = 0;
  }

  pub fn beatmap(&self) -> &Beatmap {
    return &self.beatmap;
  }

  pub fn has_ended(&self, time: Time, audio: &GameAudio) -> bool {
    return time >= audio.length() + audio.lead_out;
  }

  /// You should call this method in a loop until it returns `false`. Returns `true` if a miss has occured.
  pub fn process_miss(&mut self, time: Time) -> bool {
    // Skip unhit (if any) until we find the next hit object that can be hit.
    if let Some(hit_object) = self.beatmap.hit_objects.get(self.current_circle) {
      let hit_window_end_time = hit_object.time + self.hit_window_150;

      if hit_window_end_time >= time {
        return false;
      }

      // Unhit hit object which can not be hit anymore counts as a miss.
      self.current_circle += 1;

      return true;
    }

    return false;
  }

  pub fn hit(&mut self, time: Time, input: TaikoInput) -> Option<(HitResult, usize)> {
    if let Some(obj) = self.beatmap.hit_objects.get(self.current_circle) {
      if let Some(result) = check_hit(time, obj, input, self.hit_window_150, self.hit_window_300) {
        let hit_idx = self.current_circle;

        self.current_circle += 1;

        return Some((result, hit_idx));
      }
    }

    return None;
  }

  pub fn skip_break(&mut self, audio: &mut GameAudio, break_leniency_end: Time) {
    let time = audio.position();
    match self.is_break(time, break_leniency_end) {
      BreakState::Break(break_point) => {
        audio.set_playing(false);
        audio.set_position(break_point.end - break_leniency_end);
        audio.set_playing(true);
      }

      BreakState::Intro(break_point) => {
        audio.set_playing(false);
        audio.set_position(break_point.end - break_leniency_end);
        audio.set_playing(true);
      }

      BreakState::None => {}
    }
  }

  pub fn is_break(&mut self, time: Time, break_leniency_end: Time) -> BreakState {
    let Some(obj) = self.beatmap.hit_objects.first() else {
      return BreakState::None;
    };

    if time < obj.time - break_leniency_end && obj.time > Time::from_seconds(10.0) {
      return BreakState::Intro(BreakPoint { start: Time::zero(), end: obj.time });
    } else {
      for break_point in &self.beatmap.break_points[self.current_break_point ..] {
        if time >= break_point.end {
          self.current_break_point += 1;
        }

        if time >= break_point.start && time < break_point.end - break_leniency_end {
          return BreakState::Break(break_point.clone());
        }
      }

      return BreakState::None;
    }
  }

  pub fn hit_window_150(&self) -> Time {
    return self.hit_window_150;
  }

  pub fn hit_window_300(&self) -> Time {
    return self.hit_window_300;
  }
}
