use egui::ImageSource;

pub struct BackgroundComponent {
  pub image: egui::Image<'static>,
}

impl BackgroundComponent {
  pub fn new(source: impl Into<ImageSource<'static>>) -> Self {
    let image = egui::Image::new(source).tint(egui::Color32::from_gray(80));

    return Self { image };
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui) {
    let mut rect = ui.available_rect_before_wrap();
    let img_size = self.image.load_and_calc_size(ui, egui::Vec2::INFINITY);
    if let Some(img_size) = img_size {
      let img_aspect = img_size.x / img_size.y;
      let scr_aspect = rect.width() / rect.height();

      let width = rect.height() * img_aspect;
      let height = rect.width() / img_aspect;

      #[rustfmt::skip]
      if scr_aspect < img_aspect { rect.set_width(width);   }
      else                       { rect.set_height(height); };
    }

    self.image.paint_at(ui, rect);
  }

  pub fn set_image(&mut self, source: impl Into<ImageSource<'static>>) {
    self.image = egui::Image::new(source).rounding(6.0);
  }
}
