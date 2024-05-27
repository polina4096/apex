use std::path::{Path, PathBuf};

use fxhash::FxBuildHasher;
use indexmap::IndexMap;
use log::warn;

#[derive(Debug, Clone)]
pub struct BeatmapInfo {
  pub title: String,
  pub artist: String,
  pub creator: String,
  pub difficulty: String,
  pub bg_path: PathBuf,
}

impl<T: AsRef<str>> From<T> for BeatmapInfo {
  fn from(data: T) -> Self {
    let data = data.as_ref();
    let mut beatmap_info = Self {
      title: String::new(),
      artist: String::new(),
      creator: String::new(),
      difficulty: String::new(),
      bg_path: PathBuf::new(),
    };

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
          if line.contains(".jpg")
          || line.contains(".jpeg")
          || line.contains(".png") {
            let Some(bg_path) = line.split("\"").nth(1) else { continue };
            beatmap_info.bg_path = bg_path.into();
          }
        }

        Some("[Metadata]") => {
          let mut parts = line.split(':');
          let Some(key) = parts.next() else { continue };

          match key {
            "Title"   => if let Some(x) = parts.next() { x.clone_into(&mut beatmap_info.title); }
            "Artist"  => if let Some(x) = parts.next() { x.clone_into(&mut beatmap_info.artist); }
            "Creator" => if let Some(x) = parts.next() { x.clone_into(&mut beatmap_info.creator); }
            "Version" => if let Some(x) = parts.next() { x.clone_into(&mut beatmap_info.difficulty); }

            _ => { }
          }
        }

        _ => { }
      }

    }

    return beatmap_info;
  }
}

pub struct BeatmapCache {
  cache: IndexMap<PathBuf, BeatmapInfo, FxBuildHasher>,
}

impl BeatmapCache {
  pub fn new() -> Self {
    return Self {
      cache: IndexMap::default(),
    };
  }
  pub fn load_beatmaps(&mut self, path: impl AsRef<Path>) {
    let path = path.as_ref();
    let Ok(iter) = std::fs::read_dir(path) else {
      warn!("Failed to read directory: {:?}", path);
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

  pub fn iter(&self) -> impl Iterator<Item = (&PathBuf, &BeatmapInfo)>{
    return self.cache.iter();
  }
}
