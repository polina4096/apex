use std::{
  path::{Path, PathBuf},
  str::FromStr,
};

use indexmap::IndexMap;
use instant::Instant;
use log::warn;

use apex_framework::time::time::Time;

use super::beatmap::{Beatmap, BeatmapHash};

#[derive(Debug, Default, Clone)]
pub struct BeatmapInfo {
  pub title: String,
  pub artist: String,
  pub creator: String,
  pub variant: String,
  pub preview_time: u64,

  pub difficulty: f64,
  pub object_count: usize,
  pub length: Time,
  pub bpm: f64,

  pub hp_drain: f32,
  pub overall_difficulty: f32,

  pub file_path: PathBuf,
  pub audio_path: PathBuf,
  pub bg_path: PathBuf,
}

impl BeatmapInfo {
  pub fn from_path(path: impl AsRef<Path>) -> Self {
    let data = std::fs::read_to_string(path.as_ref()).unwrap();
    return Self::parse(data, path.as_ref().to_owned());
  }

  pub fn parse<T: AsRef<str>>(data: T, file_path: PathBuf) -> Self {
    let data = data.as_ref();
    let mut beatmap_info = Self {
      title: String::new(),
      artist: String::new(),
      creator: String::new(),
      variant: String::new(),
      preview_time: 0,

      difficulty: 0.0,
      object_count: 0,
      length: Time::zero(),
      bpm: 0.0,

      hp_drain: 0.0,
      overall_difficulty: 0.0,

      file_path,
      audio_path: PathBuf::new(),
      bg_path: PathBuf::new(),
    };

    let r_beatmap = rosu_pp::Beatmap::from_str(data).unwrap();
    let r_diff_attrs = rosu_pp::Difficulty::new().calculate(&r_beatmap);
    beatmap_info.difficulty = r_diff_attrs.stars();
    beatmap_info.bpm = r_beatmap.bpm();
    beatmap_info.object_count = r_beatmap.hit_objects.len();
    beatmap_info.hp_drain = r_beatmap.hp;
    beatmap_info.overall_difficulty = r_beatmap.od;

    // TODO: fix this properly
    beatmap_info.length =
      Time::from_seconds(r_beatmap.hit_objects.last().map(|x| x.start_time / 1000.0).unwrap_or(0.0));

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

        Some("[General]") => {
          let mut parts = line.split(':');
          let Some(key) = parts.next() else {
            continue;
          };

          #[rustfmt::skip]
          match key {
            "AudioFilename" => if let Some(x) = parts.next() { beatmap_info.audio_path = PathBuf::from(x.trim()); }
            "PreviewTime"   => if let Some(x) = parts.next() { beatmap_info.preview_time = x.trim().parse().unwrap_or(0); }

            _ => {}
          };
        }

        Some("[Metadata]") => {
          let Some((key, value)) = line.split_once(':') else {
            continue;
          };

          #[rustfmt::skip]
          match key {
            "Title"   => value.trim().clone_into(&mut beatmap_info.title),
            "Artist"  => value.trim().clone_into(&mut beatmap_info.artist),
            "Creator" => value.trim().clone_into(&mut beatmap_info.creator),
            "Version" => value.trim().clone_into(&mut beatmap_info.variant),

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
  cache: IndexMap<BeatmapHash, BeatmapInfo, ahash::RandomState>,
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

      let entry_path = entry.path();

      if entry_path.is_file() {
        if let Some(extension) = entry_path.extension() {
          if extension == "osu" {
            let data = std::fs::read_to_string(&entry_path).unwrap();
            let beatmap = Beatmap::parse(&data, path.to_owned());
            let beatmap_info = BeatmapInfo::parse(&data, entry_path.to_owned());
            self.cache.insert(beatmap.hash(), beatmap_info);
          }
        }
      }
    }
  }

  pub fn get(&self, hash: BeatmapHash) -> Option<&BeatmapInfo> {
    return self.cache.get(&hash);
  }

  pub fn get_index(&self, idx: usize) -> Option<(BeatmapHash, &BeatmapInfo)> {
    return self.cache.get_index(idx).map(|(hash, beatmap_info)| (*hash, beatmap_info));
  }

  pub fn iter(&self) -> impl Iterator<Item = (BeatmapHash, &BeatmapInfo)> {
    return self.cache.iter().map(|(hash, beatmap_info)| (*hash, beatmap_info));
  }

  pub fn last_update(&self) -> Instant {
    return self.last_update;
  }

  pub fn len(&self) -> usize {
    return self.cache.len();
  }

  pub fn is_empty(&self) -> bool {
    return self.cache.is_empty();
  }
}
