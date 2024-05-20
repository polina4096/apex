use std::path::PathBuf;

use egui::Widget;
use tap::Tap;

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
      let mut frame = egui::Frame::none()
        .stroke(egui::Stroke::new(2.0, egui::Color32::from_gray(80)))
        .outer_margin(egui::Margin { bottom: 6.0, right: 8.0, ..Default::default() }.tap_mut(|margin| {
          if selected {
            margin.left = -24.0;
            margin.right = 24.0;
          }
        }))
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
          .tint(egui::Color32::from_gray(80));

        {
          let mut rect = ui.available_rect_before_wrap();

          // ui.set_clip_rect(rect);

          if let Some(img_size) = img.load_and_calc_size(ui, egui::Vec2::INFINITY) {
            // let img_aspect = img_size.x / img_size.y;
            // let scr_aspect = rect.width() / rect.height();

            // let width = rect.height() * img_aspect;
            // let height = rect.width() / img_aspect;

            // if scr_aspect < img_aspect {
            //   rect.set_width(width);
            // } else {
            //   rect.set_height(height);
            // }

            img.paint_at(ui, rect);
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

      let response = frame.allocate_space(ui);

      // if selected {
      //   frame.frame.fill = egui::Color32::from_gray(10);
      // } else if response.hovered() {
      //   frame.frame.fill = egui::Color32::from_gray(30);
      // }

      frame.paint(ui);
    }).response;
  }
}
