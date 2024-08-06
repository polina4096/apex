use std::path::PathBuf;

use crate::{
  client::{client::Client, gameplay::beatmap_cache::BeatmapCache, ui::recording_panel::RecordingPanelView},
  core::{core::Core, graphics::video_exporter::VideoExporterConfig},
};

pub struct RecordingScreen {
  recording_panel: RecordingPanelView,
  exporter_config: VideoExporterConfig,
  beatmap_path: PathBuf,
}

impl RecordingScreen {
  pub fn new() -> Self {
    let recording_panel = RecordingPanelView::new();
    let exporter_config = VideoExporterConfig::default();
    let beatmap_path = PathBuf::new();

    return Self {
      recording_panel,
      exporter_config,
      beatmap_path,
    };
  }

  pub fn prepare(&mut self, core: &Core<Client>, beatmap_idx: usize, beatmap_cache: &BeatmapCache) {
    if let Some(path) = beatmap_cache.get_index(beatmap_idx).map(|x| x.0) {
      if self.beatmap_path != *path {
        self.beatmap_path = path.clone();
      }
    }

    self.exporter_config = VideoExporterConfig::default();
    self.recording_panel.prepare(
      core.egui.ctx(),
      &core.graphics.adapter,
      core.graphics.config.format,
      &self.beatmap_path,
      beatmap_cache,
      &mut self.exporter_config,
    );
  }

  pub fn is_open(&self) -> bool {
    return self.recording_panel.is_open;
  }

  pub fn toggle(&mut self) {
    self.recording_panel.is_open = !self.recording_panel.is_open;
  }
}
