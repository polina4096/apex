use std::path::PathBuf;

use ahash::AHashMap;
use intbits::Bits;
use log::warn;

use apex_framework::time::time::Time;

use super::taiko_hit_object::{TaikoColor, TaikoHitObject};

#[derive(Debug, Clone)]
pub struct TimingPoint {
  pub time: Time,
  pub bpm: f64,
}

impl Default for TimingPoint {
  fn default() -> Self {
    return Self { time: Time::zero(), bpm: 60.0 };
  }
}

#[derive(Debug, Clone)]
pub struct VelocityPoint {
  pub time: Time,
  pub velocity: f64,
}

impl Default for VelocityPoint {
  fn default() -> Self {
    return Self { time: Time::zero(), velocity: 1.0 };
  }
}

#[derive(Debug, Clone)]
pub struct BreakPoint {
  pub start: Time,
  pub end: Time,
}

#[derive(Clone)]
pub struct Beatmap {
  pub hit_objects: Vec<TaikoHitObject>,
  pub timing_points: Vec<TimingPoint>,
  pub velocity_points: Vec<VelocityPoint>,
  pub break_points: Vec<BreakPoint>,

  pub overall_difficulty: f32,
  pub velocity_multiplier: f32,

  pub audio: PathBuf,
}

impl Default for Beatmap {
  fn default() -> Self {
    return Self {
      hit_objects: Vec::new(),
      timing_points: Vec::new(),
      velocity_points: Vec::new(),
      break_points: Vec::new(),

      overall_difficulty: 5.0,
      velocity_multiplier: 0.6,

      audio: PathBuf::new(),
    };
  }
}

pub fn calc_hit_window_150(od: f32) -> Time {
  return Time::from_ms(if od <= 5.0 { 120.0 - 8.0 * od } else { 110.0 - 6.0 * od });
}

pub fn calc_hit_window_300(od: f32) -> Time {
  return Time::from_ms(50.0 - 3.0 * od);
}

impl Beatmap {
  pub fn parse<T: AsRef<str>>(data: T) -> Self {
    let data = data.as_ref();
    let mut objects = Vec::<TaikoHitObject>::new();

    let mut timing_points = Vec::<TimingPoint>::new();
    let mut velocity_points = Vec::<VelocityPoint>::new();
    let mut break_points = Vec::<BreakPoint>::new();

    let mut property_map = AHashMap::<&str, AHashMap<&str, &str>>::new();
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
          if line.trim().is_empty() {
            continue;
          }

          let mut parts = line.split(','); // TODO: add proper error handling
          let Some(time_ms) = parts.next().and_then(|x| x.parse::<f64>().ok()) else {
            warn!("Failed to parse timing point time at line {}", i);
            continue;
          };
          let Some(beat_length) = parts.next().and_then(|x| x.parse::<f64>().ok()) else {
            warn!("Failed to parse timing point beat length at line {}", i);
            continue;
          };
          let Some(uninherited) = parts.nth(4).map(|x| x == "1") else {
            warn!("Failed to parse timing point uninherited flag at line {}", i);
            continue;
          };

          if uninherited {
            timing_points.push(TimingPoint {
              time: Time::from_ms(time_ms),
              bpm: (60.0 * 1000.0) / beat_length,
            });

            velocity_points.push(VelocityPoint {
              time: Time::from_ms(time_ms),
              velocity: 1.0,
            });
          } else {
            velocity_points.push(VelocityPoint {
              time: Time::from_ms(time_ms),
              velocity: -100.0 / beat_length,
            });
          }
        }

        Some("[Events]") => {
          let mut parts = line.split(',');
          let Some(event_type) = parts.next() else {
            warn!("Failed to parse event type at line {}", i);
            continue;
          };

          match event_type {
            "2" | "Break" => {
              let Some(start) = parts.next().and_then(|x| x.parse::<f64>().ok()) else {
                warn!("Failed to parse break start time at line {}", i);
                continue;
              };

              let Some(end) = parts.next().and_then(|x| x.parse::<f64>().ok()) else {
                warn!("Failed to parse break end time at line {}", i);
                continue;
              };

              break_points.push(BreakPoint {
                start: Time::from_ms(start),
                end: Time::from_ms(end),
              });
            }

            _ => {}
          }
        }

        Some("[HitObjects]") => {
          let mut parts = line.split(',');
          let Some(time_in_ms) = parts.nth(2).and_then(|x| x.parse::<f64>().ok()) else {
            warn!("Failed to parse hit object time at line {}", i);
            continue;
          };

          let Some(object_type) = parts.next().and_then(|x| x.parse::<u8>().ok()) else {
            warn!("Failed to parse hit object type at line {}", i);
            continue;
          };

          let Some(hitsounds) = parts.next().and_then(|x| x.parse::<u8>().ok()) else {
            warn!("Failed to parse hit object hitsouns at line {}", i);
            continue;
          };

          // Circles
          if object_type.bit(0) {
            // Whistles or claps become kat
            if hitsounds.bit(1) || hitsounds.bit(3) {
              objects.push(TaikoHitObject {
                time: Time::from_ms(time_in_ms),
                color: TaikoColor::Kat,
                big: hitsounds.bit(2),
              });
            }
            // Everything else becomes don
            else {
              objects.push(TaikoHitObject {
                time: Time::from_ms(time_in_ms),
                color: TaikoColor::Don,
                big: hitsounds.bit(2),
              });
            }
          }
          // Sliders
          else if object_type.bit(1) {
            /* sliders here */
          }
          // Spinners
          else if object_type.bit(3) {
            /* spinners here */
          }
        }

        Some(category) => {
          if line.trim().is_empty() {
            continue;
          }

          let mut parts = line.split(':');
          let Some(key) = parts.next() else { continue };
          let Some(value) = parts.next() else { continue };

          property_map.entry(category).or_default().insert(key.trim(), value.trim());
        }

        None => {}
      }
    }

    if let Some(p) = timing_points.first() {
      if p.time != Time::zero() {
        // In osu!, if there are objects before the first timing point
        // they act as if affected by the first timing point after them
        timing_points.insert(0, TimingPoint { time: Time::zero(), bpm: p.bpm });
      }
    } else {
      timing_points.insert(0, TimingPoint::default());
    }

    if let Some(p) = velocity_points.first() {
      if p.time != Time::zero() {
        // The same above does not apply to velocity points :)
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
      break_points,
      overall_difficulty: property_map["[Difficulty]"]["OverallDifficulty"].parse().expect("idk default OD, plz fix"),
      velocity_multiplier: property_map["[Difficulty]"]["SliderMultiplier"].parse().unwrap_or(0.6),
      audio: PathBuf::from(property_map["[General]"]["AudioFilename"]),
    };
  }
}
