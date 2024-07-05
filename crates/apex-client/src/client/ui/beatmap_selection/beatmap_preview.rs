use std::num::{NonZero, NonZeroU32};

use crate::{
  client::{
    gameplay::beatmap::Beatmap,
    graphics::taiko_renderer::taiko_renderer::{TaikoRenderer, TaikoRendererConfig},
    settings::settings::Settings,
  },
  core::{
    graphics::{egui::EguiContext, graphics::Graphics},
    time::time::Time,
  },
};

pub const PREVIEW_HEIGHT: u32 = 160;

pub struct BeatmapPreviewCallback {
  time: Time,
  new_width: Option<NonZero<u32>>,
}

impl egui_wgpu::CallbackTrait for BeatmapPreviewCallback {
  fn prepare(
    &self,
    _device: &wgpu::Device,
    queue: &wgpu::Queue,
    _screen_descriptor: &egui_wgpu::ScreenDescriptor,
    _egui_encoder: &mut wgpu::CommandEncoder,
    resources: &mut egui_wgpu::CallbackResources,
  ) -> Vec<wgpu::CommandBuffer> {
    let resources: &mut TaikoRenderer = resources.get_mut().unwrap();

    if let Some(width) = self.new_width {
      resources.resize(queue, width.get(), PREVIEW_HEIGHT);
    }

    resources.prepare(queue, self.time);

    Vec::new()
  }

  fn paint<'a>(
    &self,
    _info: egui::PaintCallbackInfo,
    render_pass: &mut wgpu::RenderPass<'a>,
    resources: &'a egui_wgpu::CallbackResources,
  ) {
    let resources: &TaikoRenderer = resources.get().unwrap();

    resources.render(render_pass);
  }
}

pub struct BeatmapPreview {
  prev_width: u32,
  hit_pos: f32,
}

impl BeatmapPreview {
  pub fn new(graphics: &Graphics, egui_ctx: &mut EguiContext, settings: &Settings) -> Self {
    let beatmap_preview = Self {
      prev_width: 0,
      hit_pos: PREVIEW_HEIGHT as f32 / 2.0,
    };

    // Because the graphics pipeline must have the same lifetime as the egui render pass,
    // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
    // `paint_callback_resources` type map, which is stored alongside the render pass.
    egui_ctx.renderer.callback_resources.insert(TaikoRenderer::new(
      &graphics.device,
      &graphics.queue,
      graphics.config.format,
      TaikoRendererConfig {
        width: graphics.size.width,
        height: graphics.size.height,
        scale_factor: graphics.scale,
        scale: 0.425,
        zoom: settings.taiko.zoom(),
        hit_position_x: beatmap_preview.hit_pos / graphics.scale as f32,
        hit_position_y: beatmap_preview.hit_pos / graphics.scale as f32,
        don: settings.taiko.don_color(),
        kat: settings.taiko.kat_color(),
      },
    ));

    return beatmap_preview;
  }

  pub fn prepare(&mut self, ui: &mut egui::Ui, time: Time) {
    egui::Frame::canvas(ui.style())
      .outer_margin(egui::Margin::symmetric(12.0, 0.0))
      // .inner_margin(egui::Margin::ZERO)
      .rounding(6.0)
      .show(ui, |ui| {
        let width = ui.available_width();
        let rect = ui.allocate_space(egui::vec2(width, PREVIEW_HEIGHT as f32)).1;

        ui.painter().circle_stroke(
          rect.min + egui::vec2(self.hit_pos, self.hit_pos),
          56.0,
          egui::Stroke::new(2.0, egui::Color32::from_gray(100)),
        );

        let callback = egui_wgpu::Callback::new_paint_callback(
          rect,
          BeatmapPreviewCallback {
            time,
            new_width: if self.prev_width != width.ceil() as u32 {
              self.prev_width = width.ceil() as u32;

              Some(NonZeroU32::new(self.prev_width).unwrap_or(unsafe { NonZero::new_unchecked(1) }))
            } else {
              None
            },
          },
        );

        ui.painter().add(callback);
      });
  }

  pub fn change_beatmap(&mut self, graphics: &Graphics, egui_ctx: &mut EguiContext, beatmap: &Beatmap) {
    let resources: &mut TaikoRenderer = egui_ctx.renderer.callback_resources.get_mut().unwrap();
    resources.load_beatmap(&graphics.device, beatmap.clone());
    resources.set_hit_all(&graphics.queue);
  }
}
