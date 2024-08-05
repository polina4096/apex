use kiam::when;

use crate::{client::ui::ingame_overlay::HitResult, core::time::time::Time};

use super::{
  beatmap::{calc_hit_window_150, calc_hit_window_300},
  taiko_hit_object::{TaikoColor, TaikoHitObject},
};

/// Player can hit a circle with either the inner or outer side of the drum, this
/// enum represents exactly how the player hit the drum. See [`TaikoInput`] for
/// a more general representation of player input irregarless player input.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TaikoPlayerInput {
  DonOne,
  KatOne,
  DonTwo,
  KatTwo,
}

/// Multiple gameplay actions can map to a single input kind. In taiko a player
/// can hit a circle with either the inner or outer side of the drum. Both
/// actions lead to a single input kind.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TaikoInput {
  #[default]
  Don,
  Kat,
}

/// Logcial actions that a player can perform while playing taiko.
pub struct TaikoPlayer {
  pub current_circle: usize,
}

impl TaikoPlayer {
  pub fn new() -> Self {
    return Self { current_circle: 0 };
  }

  /// Reset the player's state to default, call when starting a new play.
  pub fn reset(&mut self) {
    self.current_circle = 0;
  }

  pub fn tick(
    &mut self,
    curr_time: Time,
    overall_difficulty: f32,
    hit_objects: &[TaikoHitObject],
    mut on_miss: impl FnMut(usize),
  ) {
    let tolerance = calc_hit_window_150(overall_difficulty);

    // Skip unhit circles until we find the next circle that should be hit.
    while let Some(circle) = hit_objects.get(self.current_circle) {
      let time = circle.time.to_ms() + tolerance.to_ms();
      if time > curr_time.to_ms() {
        break;
      }

      on_miss(self.current_circle);
      self.current_circle += 1;
    }
  }

  pub fn hit(
    &mut self,
    hit_time: Time,
    input: TaikoPlayerInput,
    overall_difficulty: f32,
    hit_objects: &[TaikoHitObject],
    on_hit: impl FnOnce(HitResult, usize, i64),
  ) {
    let hit_window_300 = calc_hit_window_300(overall_difficulty);
    let hit_window_150 = calc_hit_window_150(overall_difficulty);
    let tolerance = hit_window_150;

    // Check if the hit was within the hit window of the current circle.
    if let Some(circle) = hit_objects.get(self.current_circle) {
      let time = circle.time.to_ms();
      let hit_delta = time - hit_time.to_ms();

      if hit_delta.abs() < tolerance.to_ms() {
        if circle.color == TaikoColor::Don && (input != TaikoPlayerInput::DonOne && input != TaikoPlayerInput::DonTwo) {
          return;
        }

        if circle.color == TaikoColor::Kat && (input != TaikoPlayerInput::KatOne && input != TaikoPlayerInput::KatTwo) {
          return;
        }

        self.current_circle += 1;

        when! {
          hit_delta.abs() < hit_window_300.to_ms() => {
            on_hit(HitResult::Hit300, self.current_circle, hit_delta);
          },

          hit_delta.abs() < hit_window_150.to_ms() => {
            on_hit(HitResult::Hit150, self.current_circle, hit_delta);
          },

          _ => {
            on_hit(HitResult::Miss, self.current_circle, hit_delta);
          }
        }
      }
    }
  }
}
