use std::path::Path;

use egui::Widget;

use crate::client::gameplay::beatmap_cache::BeatmapInfo;

pub struct BeatmapCard {
  preview: egui::Image<'static>,
  title: egui::RichText,
}

impl BeatmapCard {
  pub fn new(path: &Path, info: &BeatmapInfo) -> Self {
    let bg = path.parent().unwrap().join(&info.bg_path);
    let bg = format!("file://{}", bg.to_str().unwrap());
    let preview = egui::Image::new(bg).rounding(6.0);

    let title = egui::RichText::new(format!("{} {} {}", info.artist, info.title, info.variant)).strong();

    return Self { preview, title };
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui, selected: bool) -> egui::Response {
    return ui
      .with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
        egui::Frame::window(ui.style())
          .inner_margin(egui::Margin::ZERO)
          .stroke(if selected {
            egui::Stroke::new(2.0, egui::Color32::from_gray(200))
          } else {
            egui::Stroke::new(2.0, egui::Color32::from_gray(80))
          })
          .outer_margin(egui::Margin { bottom: 6.0, ..Default::default() })
          .show(ui, |ui| {
            ui.set_height(64.0);

            // Safety: replace_with on a budget.
            //
            // We promise to return the value we read from this pointer.
            // No panics or unwinds should happen between the read and write.
            unsafe {
              let img = std::ptr::read(&self.preview)
                .tint(if selected { egui::Color32::from_gray(128) } else { egui::Color32::from_gray(60) })
                .rounding(6.0);

              std::ptr::write(&mut self.preview, img);
            }

            {
              let rect = ui.available_rect_before_wrap();

              ui.set_clip_rect(rect);

              if let Some(img_size) = self.preview.load_and_calc_size(ui, egui::Vec2::INFINITY) {
                let ratio_1 = rect.height() / rect.width();
                let ratio_2 = img_size.y / img_size.x;

                let ratio = ratio_1 / ratio_2;
                let offset = (1.0 - ratio) / 2.0;

                // Safety: replace_with on a budget.
                //
                // We promise to return the value we read from this pointer.
                // No panics or unwinds should happen between the read and write.
                unsafe {
                  let img = std::ptr::read(&self.preview)
                    .uv(egui::Rect::from_min_max(egui::pos2(0.0, offset), egui::pos2(1.0, offset + ratio)));

                  std::ptr::write(&mut self.preview, img);
                }

                self.preview.paint_at(ui, rect);
              }
            }

            egui::Frame::none().inner_margin(egui::Margin::same(8.0)).show(ui, |ui| {
              egui::Label::new(self.title.clone()).selectable(false).ui(ui);
            });
          });
      })
      .response;
  }
}
