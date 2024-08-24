use std::path::{Path, PathBuf};

use ahash::AHashMap;
use intbits::Bits;
use log::warn;

use apex_framework::time::time::Time;
use smart_default::SmartDefault;

use super::taiko_hit_object::{TaikoColor, TaikoHitObject};

// Beatmap timing-related structs
#[derive(SmartDefault, Debug, Clone)]
pub struct TimingPoint {
  #[default(Time::zero())]
  pub time: Time,

  #[default = 60.0]
  pub bpm: f64,
}

#[derive(SmartDefault, Debug, Clone)]
pub struct VelocityPoint {
  #[default(Time::zero())]
  pub time: Time,

  #[default = 1.0]
  pub velocity: f64,
}

#[derive(Debug, Clone)]
pub struct BreakPoint {
  pub start: Time,
  pub end: Time,
}

/// Unique local identifier for a beatmap.
///
/// May change when the beatmap is changed, usually does not change with new game versions. Used to differentiate
/// beatmaps in client code, most commonly in the beatmap cache and in other clientside beatmap logic. Never rely on
/// this id to uniquely identify a beatmap across multiple game clients! If needed, use online ids with hash checks, or
/// anything else that is guaranteed to be invalidated on any changes to the file and allows updates.
///
/// We can't rely on the beatmap's path because it can be changed by the user or between game versions easily without
/// affecting the beatmap itself. Also we can't go with random ids because they change every restart if not stored in
/// the beatmap files (which is a bad idea too).
///
/// For that reason, we use a blake3 hash of certain beatmap data. Generally, the possible kinds of relevant beatmap
/// data should not change between game versions which means we, for the most part, can safely rely on it's hash as a
/// unique identifier. As a rule of thumb, we should not hash empty or default values, so the hashes are not affected
/// by new fields being added.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BeatmapHash(blake3::Hash);

impl ToString for BeatmapHash {
  fn to_string(&self) -> String {
    return format!("{}", self.0);
  }
}

impl Default for BeatmapHash {
  fn default() -> Self {
    let beatmap = Beatmap::default();
    return beatmap.hash();
  }
}

// TODO: getters, hide hash field
#[derive(Clone)]
pub struct Beatmap {
  pub hit_objects: Vec<TaikoHitObject>,
  pub timing_points: Vec<TimingPoint>,
  pub velocity_points: Vec<VelocityPoint>,
  pub break_points: Vec<BreakPoint>,

  pub title: String,
  pub artist: String,
  pub creator: String,
  pub variant: String,

  pub hp_drain_rate: f32,
  pub overall_difficulty: f32,

  pub velocity_multiplier: f32,

  pub file_path: PathBuf,
  pub audio_path: PathBuf,
  pub bg_path: PathBuf,

  pub hash: Option<BeatmapHash>,
}

impl Default for Beatmap {
  fn default() -> Self {
    return Self {
      hit_objects: Vec::new(),

      timing_points: Vec::new(),
      velocity_points: Vec::new(),
      break_points: Vec::new(),

      title: String::new(),
      artist: String::new(),
      creator: String::new(),
      variant: String::new(),

      hp_drain_rate: 5.0,
      overall_difficulty: 5.0,

      velocity_multiplier: 0.6,

      file_path: PathBuf::new(),
      audio_path: PathBuf::new(),
      bg_path: PathBuf::new(),

      hash: None,
    };
  }
}

impl Beatmap {
  pub fn hash(&self) -> BeatmapHash {
    return self.hash.unwrap_or_else(|| {
      let mut hasher = blake3::Hasher::new();

      for hit_object in &self.hit_objects {
        hasher.update(&hit_object.time.to_seconds().to_le_bytes());
        hasher.update(&[hit_object.color.is_kat() as u8]);
        hasher.update(&[hit_object.big as u8]);
      }

      for timing_point in &self.timing_points {
        hasher.update(&timing_point.time.to_seconds().to_le_bytes());
        hasher.update(&timing_point.bpm.to_le_bytes());
      }

      for velocity_point in &self.velocity_points {
        hasher.update(&velocity_point.time.to_seconds().to_le_bytes());
        hasher.update(&velocity_point.velocity.to_le_bytes());
      }

      for break_point in &self.break_points {
        hasher.update(&break_point.start.to_seconds().to_le_bytes());
        hasher.update(&break_point.end.to_seconds().to_le_bytes());
      }

      hasher.update(&self.overall_difficulty.to_le_bytes());
      hasher.update(&self.velocity_multiplier.to_le_bytes());

      return BeatmapHash(hasher.finalize());
    });
  }
}

pub fn calc_hit_window_150(od: f32) -> Time {
  return Time::from_ms(if od <= 5.0 { 120.0 - 8.0 * od } else { 110.0 - 6.0 * od });
}

pub fn calc_hit_window_300(od: f32) -> Time {
  return Time::from_ms(50.0 - 3.0 * od);
}

impl Beatmap {
  pub fn from_path(path: impl AsRef<Path>) -> Self {
    let data = std::fs::read_to_string(path.as_ref()).unwrap();
    return Self::parse(data, path.as_ref().to_owned());
  }

  pub fn parse<T: AsRef<str>>(data: T, file_path: PathBuf) -> Self {
    let data = data.as_ref();
    let mut objects = Vec::<TaikoHitObject>::new();

    let mut timing_points = Vec::<TimingPoint>::new();
    let mut velocity_points = Vec::<VelocityPoint>::new();
    let mut break_points = Vec::<BreakPoint>::new();

    let mut bg_path = PathBuf::new();
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
          if line.contains(".jpg") || line.contains(".jpeg") || line.contains(".png") {
            let Some(file_bg_path) = line.split("\"").nth(1) else {
              warn!("Failed to bg path at line {}", i);
              continue;
            };

            bg_path = file_bg_path.into();
          }

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

      title: property_map["[Metadata]"]["Title"].to_owned(),
      artist: property_map["[Metadata]"]["Artist"].to_owned(),
      creator: property_map["[Metadata]"]["Creator"].to_owned(),
      variant: property_map["[Metadata]"]["Version"].to_owned(),

      hp_drain_rate: property_map["[Difficulty]"]["HPDrainRate"].parse().unwrap_or(5.0),
      overall_difficulty: property_map["[Difficulty]"]["OverallDifficulty"].parse().unwrap_or(5.0),

      velocity_multiplier: property_map["[Difficulty]"]["SliderMultiplier"].parse().unwrap_or(0.6),

      file_path,
      audio_path: PathBuf::from(property_map["[General]"]["AudioFilename"]),
      bg_path,

      hash: None,
    };
  }
}
