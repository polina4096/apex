use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

use fxhash::FxBuildHasher;
use indexmap::IndexMap;
use instant::Instant;
use log::warn;

use crate::core::time::time::Time;

#[derive(Debug, Clone)]
pub struct BeatmapInfo {
  pub title: String,
  pub artist: String,
  pub creator: String,
  pub variant: String,
  pub bg_path: PathBuf,

  pub difficulty: f64,
  pub object_count: usize,
  pub length: Time,
  pub bpm: f64,

  pub hp_drain: f64,
  pub overall_difficulty: f64,
}

impl<T: AsRef<str>> From<T> for BeatmapInfo {
  fn from(data: T) -> Self {
    let data = data.as_ref();
    let mut beatmap_info = Self {
      title: String::new(),
      artist: String::new(),
      creator: String::new(),
      variant: String::new(),
      bg_path: PathBuf::new(),

      difficulty: 0.0,
      object_count: 0,
      length: Time::zero(),
      bpm: 0.0,

      hp_drain: 0.0,
      overall_difficulty: 0.0,
    };

    let r_beatmap = rosu_pp::Beatmap::from_str(data).unwrap();
    let r_diff_attrs = rosu_pp::Difficulty::new().calculate(&r_beatmap);
    beatmap_info.difficulty = r_diff_attrs.stars();
    beatmap_info.bpm = r_beatmap.bpm();
    beatmap_info.object_count = r_beatmap.hit_objects.len();
    beatmap_info.hp_drain = r_beatmap.hp as f64;
    beatmap_info.overall_difficulty = r_beatmap.od as f64;
    beatmap_info.length = Time::from_seconds(r_beatmap.hit_objects.last().unwrap().start_time / 1000.0);

    let mut category = None::<&str>;

    for line in data.lines() {
      if category == Some("[TimingPoints]") {
        break;
      }

      if line.starts_with('[') {
        category = Some(line);
        continue;
      }

      match category {
        Some("[Events]") => {
          if line.contains(".jpg") || line.contains(".jpeg") || line.contains(".png") {
            let Some(bg_path) = line.split("\"").nth(1) else {
              continue;
            };

            beatmap_info.bg_path = bg_path.into();
          }
        }

        Some("[Metadata]") => {
          let mut parts = line.split(':');
          let Some(key) = parts.next() else {
            continue;
          };

          #[rustfmt::skip]
          match key {
            "Title"   => if let Some(x) = parts.next() { x.clone_into(&mut beatmap_info.title);   }
            "Artist"  => if let Some(x) = parts.next() { x.clone_into(&mut beatmap_info.artist);  }
            "Creator" => if let Some(x) = parts.next() { x.clone_into(&mut beatmap_info.creator); }
            "Version" => if let Some(x) = parts.next() { x.clone_into(&mut beatmap_info.variant); }

            _ => {}
          };
        }

        _ => {}
      }
    }

    return beatmap_info;
  }
}

pub struct BeatmapCache {
  cache: IndexMap<PathBuf, BeatmapInfo, FxBuildHasher>,
  last_update: Instant,
}

impl BeatmapCache {
  pub fn new() -> Self {
    return Self {
      cache: IndexMap::default(),
      last_update: Instant::now(),
    };
  }

  pub fn load_beatmaps(&mut self, path: impl AsRef<Path>) {
    let path = path.as_ref();
    let Ok(iter) = std::fs::read_dir(path) else {
      warn!("Failed to read beatmap directory: {:?}, creating new one...", path);
      if let Err(e) = std::fs::create_dir_all(path) {
        warn!("Failed to create beatmap directory: {:?}", e);
      }

      return;
    };

    for entry in iter {
      let Ok(entry) = entry else {
        warn!("Failed to read beatmap directory: {:?}", entry);
        continue;
      };

      let path = entry.path();

      if path.is_dir() {
        self.load_dir_beatmaps(&path);
      }
    }

    self.last_update = Instant::now();
  }

  pub fn load_difficulties(&mut self, path: impl AsRef<Path>) {
    self.load_dir_beatmaps(path);

    self.last_update = Instant::now();
  }

  fn load_dir_beatmaps(&mut self, path: impl AsRef<Path>) {
    let path = path.as_ref();
    let Ok(iter) = std::fs::read_dir(path) else {
      warn!("Failed to read directory: {:?}", path);
      return;
    };

    for entry in iter {
      let Ok(entry) = entry else {
        warn!("Failed to read beatmap file: {:?}", entry);
        continue;
      };

      let path = entry.path();

      if path.is_file() {
        if let Some(extension) = path.extension() {
          if extension == "osu" {
            let data = std::fs::read_to_string(&path).unwrap();
            let beatmap_info = BeatmapInfo::from(&data);
            self.cache.insert(path, beatmap_info);
          }
        }
      }
    }
  }

  pub fn get(&self, path: &PathBuf) -> Option<&BeatmapInfo> {
    return self.cache.get(path);
  }

  pub fn get_index(&self, idx: usize) -> Option<(&PathBuf, &BeatmapInfo)> {
    return self.cache.get_index(idx);
  }

  pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, &BeatmapInfo)> {
    return self.cache.iter();
  }

  pub fn last_update(&self) -> Instant {
    return self.last_update;
  }
}
