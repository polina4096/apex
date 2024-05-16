use std::{collections::HashMap, path::PathBuf};

use intbits::Bits;
use log::warn;

use crate::core::time::time::Time;

use super::hit_object::{TaikoColor, HitObject};

pub struct TimingPoint {
  pub time : Time,
  pub bpm  : f64,
}

impl Default for TimingPoint {
  fn default() -> Self {
    return Self {
      time: Time::zero(),
      bpm: 60.0
    };
  }
}

pub struct VelocityPoint {
  pub time     : Time,
  pub velocity : f64,
}

impl Default for VelocityPoint {
  fn default() -> Self {
    return Self {
      time: Time::zero(),
      velocity: 1.0
    };
  }
}

pub struct Beatmap {
  pub hit_objects     : Vec<HitObject>,
  pub timing_points   : Vec<TimingPoint>,
  pub velocity_points : Vec<VelocityPoint>,

  pub velocity_multiplier : f32,

  pub audio : PathBuf,
}

pub enum BeatmapParseError {
}

impl<T: AsRef<str>> From<T> for Beatmap {
  fn from(data: T) -> Self {
    let data = data.as_ref();
    let mut objects = Vec::<HitObject>::new();

    let mut timing_points = Vec::<TimingPoint>::new();
    let mut velocity_points = Vec::<VelocityPoint>::new();

    let mut property_map = HashMap::<&str, HashMap<&str, &str>>::new();
    let mut current_category = None::<&str>;

    for (i, line) in data.lines().enumerate() {
      if let Some(char) = line.chars().nth(0) {
        if char == '[' {
          current_category = Some(line);

          continue;
        }
      }

      match current_category {
        Some("[TimingPoints]") => {
          if line.trim().is_empty() { continue }

          let mut parts = line.split(','); // TODO: add proper error handling
          let Some(time_ms) = parts.next().and_then(|x| x.parse::<f64>().ok()) else { warn!("Failed to parse timing point time at line {}", i); continue };
          let Some(beat_length) = parts.next().and_then(|x| x.parse::<f64>().ok()) else { warn!("Failed to parse timing point beat length at line {}", i); continue };
          let Some(uninherited) = parts.nth(4).map(|x| x == "1") else { warn!("Failed to parse timing point uninherited flag at line {}", i); continue };

          if uninherited {
            timing_points.push(TimingPoint {
              time : Time::from_ms(time_ms),
              bpm  : (60.0 * 1000.0) / beat_length,
            });
          } else {
            velocity_points.push(VelocityPoint {
              time     : Time::from_ms(time_ms),
              velocity : -100.0 / beat_length,
            });
          }
        }

        Some("[HitObjects]") => {
          let mut parts = line.split(',');
          let Some(time_in_ms) = parts.nth(2).and_then(|x| x.parse::<f64>().ok()) else { warn!("Failed to parse hit object time at line {}", i); continue };
          let Some(object_type) = parts.nth(1).and_then(|x| x.parse::<u8> ().ok()) else { warn!("Failed to parse hit object type at line {}", i); continue };

          objects.push(
            HitObject {
              time  : Time::from_ms(time_in_ms),
              color : if object_type.bit(1) || object_type.bit(3)
                           { TaikoColor::KAT }
                      else { TaikoColor::DON },
              big   : object_type.bit(2),
            }
          );
        }

        Some(category) => {
          if line.trim().is_empty() { continue }

          let mut parts = line.split(':');
          let Some(key) = parts.next() else { continue };
          let Some(value) = parts.next() else { continue };

          property_map
            .entry(category)
            .or_default()
            .insert(key.trim(), value.trim());
        }

        None => { }
      }
    }

    if let Some(p) = timing_points.first() {
      if p.time != Time::zero() {
        timing_points.insert(0, TimingPoint::default());
      }
    } else {
      timing_points.insert(0, TimingPoint::default());
    }

    if let Some(p) = velocity_points.first() {
      if p.time != Time::zero() {
        velocity_points.insert(0, VelocityPoint::default());
      }
    } else {
      velocity_points.insert(0, VelocityPoint::default());
    }

    objects.sort_by(|a, b| a.time.to_seconds().total_cmp(&b.time.to_seconds()));

    return Beatmap {
      hit_objects: objects,
      timing_points,
      velocity_points,
      velocity_multiplier: property_map["[Difficulty]"]["SliderMultiplier"].parse().unwrap_or(0.6),
      audio: PathBuf::from(property_map["[General]"]["AudioFilename"]),
    };
  }
}
