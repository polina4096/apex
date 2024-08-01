use std::fmt::Write as _;

use crate::{client::client::Client, core::core::Core};

use bytesize::ByteSize;
use egui::load::ImageLoader as _;

pub struct DebugScreen {
  buffer: String,

  is_open: bool,
}

impl DebugScreen {
  pub fn new() -> Self {
    return Self { buffer: String::new(), is_open: false };
  }

  pub fn prepare(&mut self, core: &Core<Client>) {
    let ctx = core.egui_ctx();

    let mut is_open = self.is_open;

    egui::Window::new("Debug")
      .fixed_size(egui::vec2(384.0, 512.0))
      .resizable(false)
      .collapsible(false)
      .open(&mut is_open)
      .show(ctx, |ui| {
        ui.heading("Image Cache (egui)");
        let size = ByteSize::b(core.egui_ctx.image_loader.byte_size() as u64);
        write!(self.buffer, "Cache size: {}", size).unwrap();
        ui.label(&self.buffer);
        self.buffer.clear();

        ui.horizontal(|ui| {
          if ui.button("Clear").clicked() {
            core.egui_ctx.image_loader.forget_all();
          };
        });
      });

    self.is_open = is_open;
  }

  pub fn is_open(&self) -> bool {
    return self.is_open;
  }

  pub fn toggle(&mut self) {
    self.is_open = !self.is_open;
  }
}
