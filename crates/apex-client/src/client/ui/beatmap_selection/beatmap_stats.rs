use crate::client::gameplay::beatmap_cache::BeatmapInfo;

pub struct BeatmapStats {

}

impl BeatmapStats {
  pub fn new() -> Self {
    return Self { };
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui, beatmap: &BeatmapInfo) {
    let max_width = ui.available_width();
    ui.set_width(max_width.min(640.0));
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
      egui::Frame::window(ui.style())
        .outer_margin(egui::Margin::same(12.0))
        .inner_margin(egui::Margin::symmetric(24.0, 16.0))
        .show(ui, |ui| {
          ui.label(egui::RichText::new(format!("{} - {}", beatmap.artist, beatmap.title)).size(24.0).strong());
          ui.label(egui::RichText::new(format!("{} by {}", beatmap.difficulty, beatmap.creator)).size(18.0));
        });
    });
  }
}
