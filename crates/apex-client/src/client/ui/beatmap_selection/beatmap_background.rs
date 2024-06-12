use std::{fmt::Write as _, path::Path};

pub struct BeatmapBackground {
  buffer: String,
}

impl BeatmapBackground {
  pub fn new() -> Self {
    return Self { buffer: String::new() };
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui, bg_path: &Path) {
    self.buffer.clear();

    write!(&mut self.buffer, "file://{}", bg_path.to_str().unwrap()).unwrap();

    #[rustfmt::skip]
    let img = egui::Image::new(&self.buffer)
      .tint(egui::Color32::from_gray(80));

    let mut rect = ui.available_rect_before_wrap();
    let img_size = img.load_and_calc_size(ui, egui::Vec2::INFINITY);
    if let Some(img_size) = img_size {
      let img_aspect = img_size.x / img_size.y;
      let scr_aspect = rect.width() / rect.height();

      let width = rect.height() * img_aspect;
      let height = rect.width() / img_aspect;

      #[rustfmt::skip]
      if scr_aspect < img_aspect { rect.set_width(width);   }
      else                       { rect.set_height(height); };
    }

    img.paint_at(ui, rect);
  }
}
