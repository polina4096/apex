use std::{fmt::Write as _, sync::Arc};

use crate::client::client::Client;

use apex_framework::core::Core;
use bytesize::ByteSize;
use egui::load::{BytesLoader, ImageLoader, TextureLoader};

pub struct DebugScreen {
  buffer: String,

  image_loaders: Vec<Arc<dyn ImageLoader + Send + Sync>>,
  texture_loaders: Vec<Arc<dyn TextureLoader + Send + Sync>>,
  bytes_loaders: Vec<Arc<dyn BytesLoader + Send + Sync>>,

  refresh_loaders: bool,
  is_open: bool,
}

impl DebugScreen {
  pub fn new() -> Self {
    return Self {
      buffer: String::new(),
      image_loaders: Vec::new(),
      texture_loaders: Vec::new(),
      bytes_loaders: Vec::new(),
      refresh_loaders: true,
      is_open: false,
    };
  }

  pub fn prepare(&mut self, core: &Core<Client>) {
    let ctx = core.egui.ctx();

    let mut is_open = self.is_open;

    if self.refresh_loaders {
      self.refresh_loaders = false;

      let loaders = core.egui.ctx().loaders();
      self.image_loaders = loaders.image.lock().iter().cloned().collect();
      self.texture_loaders = loaders.texture.lock().iter().cloned().collect();
      self.bytes_loaders = loaders.bytes.lock().iter().cloned().collect();
    }

    egui::Window::new("Debug")
      .fixed_size(egui::vec2(384.0, 512.0))
      .resizable(false)
      .collapsible(false)
      .open(&mut is_open)
      .show(ctx, |ui| {
        ui.heading("Caches (egui)");
        // let size = ByteSize::b(core.egui_ctx.image_loader.byte_size() as u64);
        // core.egui_ctx.image_loader.forget_all();

        let size = ByteSize::b(self.image_loaders.iter().map(|x| x.byte_size()).sum::<usize>() as u64);
        write!(self.buffer, "Image caches: {}", size).unwrap();
        ui.label(&self.buffer);
        self.buffer.clear();

        let size = ByteSize::b(self.texture_loaders.iter().map(|x| x.byte_size()).sum::<usize>() as u64);
        write!(self.buffer, "Texture caches: {}", size).unwrap();
        ui.label(&self.buffer);
        self.buffer.clear();

        let size = ByteSize::b(self.bytes_loaders.iter().map(|x| x.byte_size()).sum::<usize>() as u64);
        write!(self.buffer, "Bytes caches: {}", size).unwrap();
        ui.label(&self.buffer);
        self.buffer.clear();

        ui.horizontal(|ui| {
          ui.label("Wipe caches:");

          if ui.button("All").clicked() {
            core.egui.ctx().loaders().image.lock().iter().for_each(|x| x.forget_all());
            core.egui.ctx().loaders().texture.lock().iter().for_each(|x| x.forget_all());
            core.egui.ctx().loaders().bytes.lock().iter().for_each(|x| x.forget_all());
          };

          if ui.button("Image").clicked() {
            core.egui.ctx().loaders().image.lock().iter().for_each(|x| x.forget_all());
          };

          if ui.button("Texture").clicked() {
            core.egui.ctx().loaders().texture.lock().iter().for_each(|x| x.forget_all());
          };

          if ui.button("Bytes").clicked() {
            core.egui.ctx().loaders().bytes.lock().iter().for_each(|x| x.forget_all());
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

    if self.is_open {
      self.refresh_loaders = true;
    }
  }
}
