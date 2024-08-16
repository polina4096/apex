use apex_framework::time::time::Time;

use crate::client::gameplay::{
  taiko_hit_object::{TaikoColor, TaikoHitObject},
  taiko_player::TaikoInput,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Judgement {
  Hit300,
  Hit150,
  Miss,
}

pub struct HitResult {
  pub judgement: Judgement,

  /// Negative is early, positive is late
  pub hit_delta: Time,
}

pub fn check_hit(
  hit_time: Time,
  hit_object: &TaikoHitObject,
  hit_input: TaikoInput,
  hit_window_150: Time,
  hit_window_300: Time,
) -> Option<HitResult> {
  let obj_time = hit_object.time;
  let hit_delta = hit_time - obj_time;

  // Check if the hit was within the hit window of the current circle.
  if hit_delta.abs() < hit_window_150 {
    // Make sure the hit was on the correct side of the drum.
    if { false }
      || (hit_object.color == TaikoColor::Don
        && (hit_input != TaikoInput::DonRight && hit_input != TaikoInput::DonLeft))
      || (hit_object.color == TaikoColor::Kat
        && (hit_input != TaikoInput::KatLeft && hit_input != TaikoInput::KatRight))
    {
      return Some(HitResult { judgement: Judgement::Miss, hit_delta });
    }

    // Check if the hit was within the 300ms hit window, otherwise it's a 150.
    if hit_delta.abs() < hit_window_300 {
      return Some(HitResult { judgement: Judgement::Hit300, hit_delta });
    } else {
      // We know the hit was within the 150ms hit window, so it's a 150.
      return Some(HitResult { judgement: Judgement::Hit150, hit_delta });
    }
  }

  return None;
}
