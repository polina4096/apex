use std::path::PathBuf;

use action_bar::ActionBar;
use apex_framework::{
  core::Core,
  event::EventBus,
  graphics::{drawable::Drawable, graphics::Graphics},
  time::clock::AbstractClock,
};
use beatmap_card::BeatmapCard;
use beatmap_list::BeatmapList;
use beatmap_preview::BeatmapPreview;
use beatmap_scores::BeatmapScores;
use beatmap_stats::BeatmapStats;
use tap::Tap;

use crate::client::{
  client::Client,
  event::ClientEvent,
  gameplay::{beatmap::Beatmap, beatmap_cache::BeatmapCache, beatmap_selector::BeatmapSelector},
  score::score_cache::{ScoreCache, ScoreId},
  settings::Settings,
};

use super::background_component::BackgroundComponent;

pub mod action_bar;
pub mod beatmap_card;
pub mod beatmap_list;
pub mod beatmap_preview;
pub mod beatmap_scores;
pub mod beatmap_stats;

pub struct BeatmapSelectionView {
  prev_beatmap: PathBuf,
  score_ids: Vec<ScoreId>,

  beatmap_bg: BackgroundComponent,
  beatmap_list: BeatmapList,
  beatmap_stats: BeatmapStats,
  beatmap_preview: BeatmapPreview,
  beatmap_scores: BeatmapScores,
  action_bar: ActionBar,
}

impl BeatmapSelectionView {
  pub fn new(
    event_bus: EventBus<ClientEvent>,
    beatmap_cache: &BeatmapCache,
    clock: &mut impl AbstractClock,
    graphics: &Graphics,
    settings: &Settings,
  ) -> Self {
    let mut beatmap_cards = vec![];
    for (path, info) in beatmap_cache.iter() {
      let card = BeatmapCard::new(path, info);
      beatmap_cards.push(card);
    }

    return Self {
      prev_beatmap: PathBuf::new(),
      score_ids: Vec::new(),

      beatmap_bg: BackgroundComponent::new(""),
      beatmap_list: BeatmapList::new(event_bus.clone(), beatmap_cards),
      beatmap_stats: BeatmapStats::new(),
      beatmap_preview: BeatmapPreview::new(graphics, settings),
      beatmap_scores: BeatmapScores::new(event_bus.clone()),
      action_bar: ActionBar::new(event_bus, clock),
    };
  }

  pub fn scale(&mut self, scale_factor: f64) {
    self.beatmap_preview.scale(scale_factor);
  }

  pub fn scroll_to_selected(&mut self) {
    self.beatmap_list.scroll_to_selected();
  }

  pub fn prepare(
    &mut self,
    core: &mut Core<Client>,
    beatmap_cache: &BeatmapCache,
    score_cache: &mut ScoreCache,
    selector: &mut BeatmapSelector,
    clock: &mut impl AbstractClock,
  ) {
    selector.tick(beatmap_cache);

    use egui_extras::{Size, StripBuilder};

    let selected = selector.selected();
    let Some((path, info)) = beatmap_cache.get_index(selected) else {
      // TODO: Show error message no beatmaps found
      return;
    };

    if self.prev_beatmap != *path {
      self.prev_beatmap = path.clone();

      let bg_path = path.parent().unwrap().join(&info.bg_path);
      let bg = format!("file://{}", bg_path.to_str().unwrap());
      self.beatmap_bg = BackgroundComponent::new(bg);

      let data = std::fs::read_to_string(path).unwrap();
      let beatmap = Beatmap::parse(data);

      self.beatmap_preview.change_beatmap(&core.graphics, core.egui.renderer_mut(), &beatmap);

      self.update_scores(score_cache, path);
    }

    let (egui_ctx, egui_renderer) = core.egui.ctx_renderer_mut();
    egui::CentralPanel::default().frame(egui::Frame::none()).show(egui_ctx, |ui| {
      self.beatmap_bg.prepare(ui);

      StripBuilder::new(ui) //
        .size(Size::remainder())
        .size(Size::relative(0.4))
        .horizontal(|mut builder| {
          builder.cell(|ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
              egui::Frame::none() //
                .inner_margin(egui::Margin::same(12.0).tap_mut(|x| x.bottom = 0.0))
                .show(ui, |ui| {
                  self.beatmap_stats.prepare(ui, info);
                  ui.add_space(8.0);
                  self.beatmap_preview.prepare(ui, clock, egui_renderer);
                  ui.add_space(8.0);
                  self.beatmap_scores.prepare(ui, score_cache, &self.score_ids, path);
                });
            });

            ui.with_layout(egui::Layout::left_to_right(egui::Align::Max), |ui| {
              self.action_bar.prepare(ui, clock);
            });
          });

          builder.cell(|ui| {
            self.beatmap_list.prepare(ui, beatmap_cache, selector);
          });
        });
    });
  }

  pub fn update_scores(&mut self, score_cache: &mut ScoreCache, path: &PathBuf) {
    self.score_ids.clear();
    if let Some(score_ids) = score_cache.beatmap_scores(path) {
      self.score_ids.extend(score_ids.iter());
    }
  }
}

impl Drawable for BeatmapSelectionView {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    self.beatmap_preview.recreate(device, queue, format);
  }
}
