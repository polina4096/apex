use kiam::when;

use crate::{client::ui::ingame_overlay::ingame_overlay_view::HitResult, core::time::time::Time};

use super::{beatmap::Beatmap, taiko_hit_object::TaikoColor};

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
  current_circle: usize,
}

impl TaikoPlayer {
  pub fn new() -> Self {
    return Self {
      current_circle: 0,
    };
  }

  /// Reset the player's state to default, call when starting a new play.
  pub fn reset(&mut self) {
    self.current_circle = 0;
  }

  pub fn hit(&mut self, hit_time: Time, input: TaikoPlayerInput, beatmap: &Beatmap, on_hit: impl FnOnce(HitResult, usize)) {
    let audio_offset = 0.0;
    let audio_offset = Time::from_seconds(audio_offset / 1000.0);

    let od = beatmap.overall_difficulty;
    let hit_window_300 = Time::from_ms(50.0 - 3.0 * od);
    let hit_window_100 = Time::from_ms(if od <= 5.0 { 120.0 - 8.0 * od } else { 110.0 - 6.0 * od });
    let tolerance = hit_window_100;

    // Skip unhit circles until we find the next circle that should be hit.
    while let Some(circle) = beatmap.hit_objects.get(self.current_circle) {
      let time = circle.time.to_ms() as i64 + audio_offset.to_ms() as i64 + tolerance.to_ms() as i64;
      if time > hit_time.to_ms() as i64 {
        break;
      }

      self.current_circle += 1;
    }

    // Check if the hit was within the hit window of the current circle.
    if let Some(circle) = beatmap.hit_objects.get(self.current_circle) {
      let time = circle.time.to_ms() as i64 + audio_offset.to_ms() as i64;
      let hit_delta = time - hit_time.to_ms() as i64;

      if hit_delta.abs() < tolerance.to_ms() as i64 {
        if circle.color == TaikoColor::Don && (input != TaikoPlayerInput::DonOne && input != TaikoPlayerInput::DonTwo) {
          return;
        }

        if circle.color == TaikoColor::Kat && (input != TaikoPlayerInput::KatOne && input != TaikoPlayerInput::KatTwo) {
          return;
        }

        self.current_circle += 1;

        when! {
          hit_delta.abs() < hit_window_300.to_ms() as i64 => {
            on_hit(HitResult::Hit300, self.current_circle);
          },

          hit_delta.abs() < hit_window_100.to_ms() as i64 => {
            on_hit(HitResult::Hit100, self.current_circle);
          },

          _ => {
            on_hit(HitResult::Miss, self.current_circle);
          }
        }
      }
    }
  }
}
