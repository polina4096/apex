use egui::ImageSource;

pub struct CardComponent {
  pub selected: bool,
  pub image: egui::Image<'static>,
}

impl CardComponent {
  pub fn new(source: impl Into<ImageSource<'static>>) -> Self {
    let image = egui::Image::new(source).rounding(6.0);
    let selected = false;

    return Self { selected, image };
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui, height: f32, inner: impl FnOnce(&mut egui::Ui)) -> egui::Response {
    return ui
      .with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
        egui::Frame::window(ui.style())
          .inner_margin(egui::Margin::ZERO)
          .stroke(if self.selected {
            egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, 48))
          } else {
            egui::Stroke::new(2.0, egui::Color32::from_rgba_unmultiplied(160, 160, 160, 48))
          })
          .outer_margin(egui::Margin { bottom: 6.0, ..Default::default() })
          .show(ui, |ui| {
            ui.set_height(height);

            // Safety: replace_with on a budget.
            //
            // We promise to return the value we read from this pointer.
            // No panics or unwinds should happen between the read and write.
            unsafe {
              let img = std::ptr::read(&self.image)
                .tint(if self.selected { egui::Color32::from_gray(128) } else { egui::Color32::from_gray(60) })
                .rounding(6.0);

              std::ptr::write(&mut self.image, img);
            }

            {
              let rect = ui.available_rect_before_wrap();

              ui.set_clip_rect(rect);

              if let Some(img_size) = self.image.load_and_calc_size(ui, egui::Vec2::INFINITY) {
                let ratio_1 = rect.height() / rect.width();
                let ratio_2 = img_size.y / img_size.x;

                let ratio = ratio_1 / ratio_2;
                let offset = (1.0 - ratio) / 2.0;

                // Safety: replace_with on a budget.
                //
                // We promise to return the value we read from this pointer.
                // No panics or unwinds should happen between the read and write.
                unsafe {
                  let img = std::ptr::read(&self.image)
                    .uv(egui::Rect::from_min_max(egui::pos2(0.0, offset), egui::pos2(1.0, offset + ratio)));

                  std::ptr::write(&mut self.image, img);
                }

                self.image.paint_at(ui, rect);
              }
            }

            inner(ui);
          });
      })
      .response;
  }
}
