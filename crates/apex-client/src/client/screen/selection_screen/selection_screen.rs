use std::path::PathBuf;

use crate::{
  client::{
    client::Client,
    event::ClientEvent,
    gameplay::{beatmap_cache::BeatmapCache, beatmap_selector::BeatmapSelector},
    score::score_cache::ScoreCache,
    settings::Settings,
    ui::beatmap_selection::BeatmapSelectionView,
  },
  core::{
    core::Core,
    event::EventBus,
    graphics::{drawable::Drawable, egui::EguiContext, graphics::Graphics},
    time::clock::AbstractClock,
  },
};

pub struct SelectionScreen {
  beatmap_selection: BeatmapSelectionView,
  beatmap_selector: BeatmapSelector,
}

impl SelectionScreen {
  pub fn new(
    event_bus: EventBus<ClientEvent>,
    beatmap_cache: &BeatmapCache,
    clock: &mut impl AbstractClock,
    graphics: &Graphics,
    egui_ctx: &mut EguiContext,
    settings: &Settings,
  ) -> Self {
    let beatmap_selection = BeatmapSelectionView::new(event_bus, beatmap_cache, clock, graphics, egui_ctx, settings);
    let beatmap_selector = BeatmapSelector::new(beatmap_cache);

    return Self { beatmap_selection, beatmap_selector };
  }

  pub fn prepare(
    &mut self,
    core: &mut Core<Client>,
    beatmap_cache: &BeatmapCache,
    score_cache: &mut ScoreCache,
    clock: &mut impl AbstractClock,
  ) {
    self.beatmap_selection.prepare(core, beatmap_cache, score_cache, &mut self.beatmap_selector, clock);
  }

  pub fn scroll_to_selected(&mut self) {
    self.beatmap_selection.scroll_to_selected();
  }

  pub fn scale(&mut self, scale_factor: f64) {
    self.beatmap_selection.scale(scale_factor);
  }

  pub fn update_scores(&mut self, score_cache: &mut ScoreCache, path: &PathBuf) {
    self.beatmap_selection.update_scores(score_cache, path);
  }

  pub fn beatmap_selector(&self) -> &BeatmapSelector {
    return &self.beatmap_selector;
  }

  pub fn beatmap_selector_mut(&mut self) -> &mut BeatmapSelector {
    return &mut self.beatmap_selector;
  }
}

impl Drawable for SelectionScreen {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    self.beatmap_selection.recreate(device, queue, format);
  }
}
