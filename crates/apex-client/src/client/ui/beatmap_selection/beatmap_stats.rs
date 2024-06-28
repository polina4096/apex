use egui::Widget;

use crate::{
  client::{event::ClientEvent, gameplay::beatmap_cache::BeatmapInfo},
  core::event::EventBus,
};

pub struct BeatmapStats {
  event_bus: EventBus<ClientEvent>,
}

impl BeatmapStats {
  pub fn new(event_bus: EventBus<ClientEvent>) -> Self {
    return Self { event_bus };
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui, beatmap: &BeatmapInfo) {
    use egui_extras::{Size, StripBuilder};

    egui::Frame::window(ui.style())
      .outer_margin(egui::Margin::same(12.0))
      .inner_margin(egui::Margin::symmetric(24.0, 16.0))
      .show(ui, |ui| {
        ui.set_max_height(80.0);

        StripBuilder::new(ui).size(Size::remainder()).size(Size::exact(96.0)).horizontal(|mut builder| {
          builder.cell(|ui| {
            let text = format!("{} - {}", beatmap.artist, beatmap.title);
            egui::Label::new(egui::RichText::new(text).size(24.0).strong()).truncate(true).ui(ui);

            let text = format!("{} by {}", beatmap.variant, beatmap.creator);
            egui::Label::new(egui::RichText::new(text).size(18.0)).truncate(true).ui(ui);

            ui.with_layout(egui::Layout::left_to_right(egui::Align::Max), |ui| {
              let text = format!("{:.2} HP", beatmap.hp_drain);
              egui::Label::new(egui::RichText::new(text).size(16.0).weak()).truncate(true).ui(ui);

              egui::Label::new(egui::RichText::new("∙").size(16.0).weak()).truncate(true).ui(ui);

              let text = format!("{:.2} OD", beatmap.overall_difficulty);
              egui::Label::new(egui::RichText::new(text).size(16.0).weak()).truncate(true).ui(ui);
            });
          });

          builder.cell(|ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
              let text = format!("{:.2} ★", beatmap.difficulty);
              egui::Label::new(egui::RichText::new(text).size(14.0).line_height(Some(18.0))).truncate(true).ui(ui);

              let text = format!("{} ⏺", beatmap.object_count);
              egui::Label::new(egui::RichText::new(text).size(14.0).line_height(Some(18.0))).truncate(true).ui(ui);

              let text = format!("{:.2}s", beatmap.length.to_seconds());
              egui::Label::new(egui::RichText::new(text).size(14.0).line_height(Some(18.0))).truncate(true).ui(ui);

              let text = format!("{:.2} BPM", beatmap.bpm);
              egui::Label::new(egui::RichText::new(text).size(14.0).line_height(Some(18.0))).truncate(true).ui(ui);
            });
          });
        });
      });
  }
}
