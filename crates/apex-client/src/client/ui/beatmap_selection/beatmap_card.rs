use std::path::Path;

use egui::Widget;

use crate::client::{gameplay::beatmap_cache::BeatmapInfo, ui::card_component::CardComponent};

pub struct BeatmapCard {
  card: CardComponent,
  title: egui::RichText,
}

impl BeatmapCard {
  pub fn new(path: &Path, info: &BeatmapInfo) -> Self {
    let bg = path.parent().unwrap().join(&info.bg_path);
    let bg = format!("file://{}", bg.to_str().unwrap());

    let card = CardComponent::new(bg);
    let title = egui::RichText::new(format!("{} {} {}", info.artist, info.title, info.variant)).strong();

    return Self { card, title };
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui, selected: bool) -> egui::Response {
    self.card.selected = selected;

    return self.card.prepare(ui, |ui| {
      egui::Label::new(self.title.clone()).selectable(false).ui(ui);
    });
  }
}
