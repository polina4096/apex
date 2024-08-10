use std::path::PathBuf;

use apex_framework::{
  event::EventBus,
  time::{clock::AbstractClock as _, time::Time},
};
use jiff::Timestamp;

use crate::client::{
  audio::game_audio::GameAudio,
  event::ClientEvent,
  score::{
    judgement_processor::{check_hit, HitResult, Judgement},
    score_processor::ScoreProcessor,
  },
  ui::ingame_overlay::IngameOverlayView,
};

use super::beatmap::{calc_hit_window_150, calc_hit_window_300, Beatmap, BreakPoint};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaikoInput {
  DonOne = 0,
  DonTwo = 1,
  KatOne = 2,
  KatTwo = 3,
}

impl TryFrom<u8> for TaikoInput {
  type Error = ();

  fn try_from(value: u8) -> Result<Self, Self::Error> {
    match value {
      0 => Ok(TaikoInput::DonOne),
      1 => Ok(TaikoInput::DonTwo),
      2 => Ok(TaikoInput::KatOne),
      3 => Ok(TaikoInput::KatTwo),
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
  event_bus: EventBus<ClientEvent>,

  beatmap: Beatmap,
  beatmap_path: PathBuf,
  player_username: String,

  // Memoized for performance reasons.
  hit_window_150: Time,
  hit_window_300: Time,

  current_circle: usize,
  current_break_point: usize,
}

impl TaikoPlayer {
  pub fn new(username: String, event_bus: EventBus<ClientEvent>) -> Self {
    return Self {
      event_bus,
      beatmap: Beatmap::default(),
      beatmap_path: PathBuf::new(),
      player_username: username,
      hit_window_150: Time::zero(),
      hit_window_300: Time::zero(),
      current_circle: 0,
      current_break_point: 0,
    };
  }

  pub fn play(&mut self, beatmap: Beatmap, beatmap_path: PathBuf) {
    self.reset();

    self.beatmap = beatmap;
    self.beatmap_path = beatmap_path;
    self.hit_window_150 = calc_hit_window_150(self.beatmap.overall_difficulty);
    self.hit_window_300 = calc_hit_window_300(self.beatmap.overall_difficulty);
  }

  pub fn reset(&mut self) {
    self.current_circle = 0;
    self.current_break_point = 0;
  }

  pub fn tick(
    &mut self,
    time: Time,
    audio: &mut GameAudio,
    score_processor: &mut ScoreProcessor,
    ingame_overlay: &mut IngameOverlayView,
  ) {
    // Finish the play if beatmap is over.
    if time >= audio.length() + audio.lead_out {
      let path = self.beatmap_path.clone();
      let score = score_processor.export(Timestamp::now(), self.player_username.clone());
      self.event_bus.send(ClientEvent::ShowResultScreen { path, score });
    }

    // Skip unhit (if any) until we find the next hit object that can be hit.
    while let Some(hit_object) = self.beatmap.hit_objects.get(self.current_circle) {
      let hit_window_end_time = hit_object.time + self.hit_window_150;

      if hit_window_end_time >= time {
        break;
      }

      // Unhit hit object which can not be hit anymore counts as a miss.
      ingame_overlay.update_last_hit_result(Judgement::Miss);
      score_processor.feed(time, None, Judgement::Miss);

      self.current_circle += 1;
    }
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

  pub fn set_username(&mut self, username: String) {
    self.player_username = username;
  }
}
