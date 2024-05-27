use std::path::PathBuf;

use egui::Widget;

use crate::client::gameplay::beatmap_cache::BeatmapInfo;

pub struct BeatmapCard {
  pub path: PathBuf,
  pub info: BeatmapInfo,
}

impl BeatmapCard {
  pub fn new(path: PathBuf, info: BeatmapInfo) -> Self {
    return Self { path, info };
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui, selected: bool) -> egui::Response {
    return ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
      let mut frame = egui::Frame::window(ui.style())
        .inner_margin(egui::Margin::ZERO)
        .stroke(if selected {
          egui::Stroke::new(2.0, egui::Color32::from_gray(200))
        } else {
          egui::Stroke::new(2.0, egui::Color32::from_gray(80))
        })
        .outer_margin(egui::Margin { bottom: 6.0, ..Default::default() })
        .begin(ui);

      {
        let ui = &mut frame.content_ui;
        ui.set_height(64.0);

        let beatmap_str = format!(
          "{} - {} [{}]",
          self.info.artist,
          self.info.title,
          self.info.difficulty
        );

        let bg = self.path.parent().unwrap().join(&self.info.bg_path);
        let img = egui::Image::new(format!("file://{}", bg.to_str().unwrap()))
          .tint(if selected { egui::Color32::from_gray(128) } else { egui::Color32::from_gray(60) })
          .rounding(6.0);

        {
          let rect = ui.available_rect_before_wrap();

          ui.set_clip_rect(rect);

          if let Some(img_size) = img.load_and_calc_size(ui, egui::Vec2::INFINITY) {
            let ratio_1 = rect.height() / rect.width();
            let ratio_2 = img_size.y / img_size.x;

            let ratio = ratio_1 / ratio_2;
            let offset = (1.0 - ratio) / 2.0;

            img
              .uv(egui::Rect::from_min_max(
                egui::pos2(0.0, offset),
                egui::pos2(1.0, offset + ratio),
              ))
              .paint_at(ui, rect);
          }
        }

        egui::Frame::none()
          .inner_margin(egui::Margin::same(8.0))
          .show(ui, |ui| {
            egui::Label::new(egui::RichText::new(beatmap_str).strong())
              .selectable(false)
              .ui(ui);
          });
      }

      frame.paint(ui);
    }).response;
  }
}
