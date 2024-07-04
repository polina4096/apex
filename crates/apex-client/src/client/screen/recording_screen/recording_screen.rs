use std::path::PathBuf;

use pollster::FutureExt as _;
use tap::Tap as _;

use crate::{
  client::{
    client::Client,
    gameplay::beatmap_cache::BeatmapCache,
    graphics::taiko_renderer::taiko_renderer::{TaikoRenderer, TaikoRendererConfig},
    ui::recording_panel::RecordingPanelView,
  },
  core::{
    core::Core,
    graphics::{color::Color, graphics::Graphics, video_exporter::VideoExporterConfig},
  },
};

pub struct RecordingScreen {
  recording_panel: RecordingPanelView,
  exporter_config: VideoExporterConfig,

  beatmap_path: PathBuf,
}

impl RecordingScreen {
  pub fn new(graphics: &Graphics) -> Self {
    let recording_panel = RecordingPanelView::new();
    let exporter_config = VideoExporterConfig::default();

    let beatmap_path = PathBuf::new();

    return Self {
      recording_panel,
      exporter_config,
      beatmap_path,
    };
  }

  pub fn prepare(&mut self, core: &Core<Client>, beatmap_cache: &BeatmapCache) {
    self.recording_panel.prepare(
      core.egui_ctx.egui_ctx(),
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

  pub fn set_beatmap(&mut self, path: PathBuf) {
    self.beatmap_path = path;
  }
}
