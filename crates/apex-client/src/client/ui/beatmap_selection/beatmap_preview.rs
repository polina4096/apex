use std::num::{NonZero, NonZeroU32};

use apex_framework::{
  graphics::{color::Color, drawable::Drawable, graphics::Graphics},
  time::{clock::AbstractClock, time::Time},
};

use crate::client::{
  gameplay::beatmap::Beatmap,
  graphics::taiko_renderer::taiko_renderer::{TaikoRenderer, TaikoRendererConfig},
  settings::Settings,
};

pub const PREVIEW_HEIGHT: u32 = 160;

pub struct BeatmapPreviewCallback {
  time: Time,
  new_width: Option<NonZero<u32>>,
  new_scale: Option<f64>,
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
    let Some(resources) = resources.get_mut::<TaikoRenderer>() else {
      return Vec::new();
    };

    if let Some(width) = self.new_width {
      resources.resize(queue, width.get(), PREVIEW_HEIGHT);
    }

    if let Some(scale_factor) = self.new_scale {
      resources.scale(queue, scale_factor);
      resources.set_hit_position_x(queue, PREVIEW_HEIGHT as f32 / 2.0 / scale_factor as f32);
      resources.set_hit_position_y(queue, PREVIEW_HEIGHT as f32 / 2.0 / scale_factor as f32);
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
    let Some(resources) = resources.get::<TaikoRenderer>() else {
      return;
    };

    resources.render(render_pass);
  }
}

pub struct BeatmapPreview {
  prev_width: u32,
  scale_factor: f64,
  zoom: f64,
  don_color: Color,
  kat_color: Color,

  current_beatmap: Option<Beatmap>,
  new_renderer: Option<TaikoRenderer>,
  new_scale_factor: Option<f64>,

  last_state: bool,
  last_bits: u32,
}

impl BeatmapPreview {
  pub fn new(graphics: &Graphics, settings: &Settings) -> Self {
    let hit_pos = PREVIEW_HEIGHT as f32 / 2.0;
    let taiko_renderer = TaikoRenderer::new(
      &graphics.device,
      &graphics.queue,
      graphics.config.format,
      TaikoRendererConfig {
        width: graphics.size.width,
        height: PREVIEW_HEIGHT,
        scale_factor: graphics.scale,
        scale: 0.425,
        zoom: settings.taiko.zoom(),
        hit_position_x: hit_pos / graphics.scale as f32,
        hit_position_y: hit_pos / graphics.scale as f32,
        don: settings.taiko.don_color(),
        kat: settings.taiko.kat_color(),
        hit_height: 12.5,
      },
    );

    let beatmap_preview = Self {
      prev_width: 0,
      scale_factor: graphics.scale,
      zoom: settings.taiko.zoom(),
      don_color: settings.taiko.don_color(),
      kat_color: settings.taiko.kat_color(),
      current_beatmap: None,
      new_renderer: Some(taiko_renderer),
      new_scale_factor: None,
      last_state: false,
      last_bits: 0,
    };

    return beatmap_preview;
  }

  pub fn prepare(
    &mut self,
    ui: &mut egui::Ui,
    clock: &mut impl AbstractClock,
    egui_renderer: &mut egui_wgpu::Renderer,
  ) {
    let hit_pos = PREVIEW_HEIGHT as f32 / 2.0;

    egui::Frame::canvas(ui.style()) //
      .rounding(6.0)
      .show(ui, |ui| {
        let time = clock.position();
        let width = ui.available_width();
        let rect = ui.allocate_space(egui::vec2(width, PREVIEW_HEIGHT as f32)).1;

        ui.painter().circle_stroke(
          rect.min + egui::vec2(hit_pos, hit_pos),
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
            new_scale: self.new_scale_factor.take(),
          },
        );

        if let Some(taiko_renderer) = self.new_renderer.take() {
          // Because the graphics pipeline must have the same lifetime as the egui render pass,
          // instead of storing the pipeline in our `Custom3D` struct, we insert it into the
          // `paint_callback_resources` type map, which is stored alongside the render pass.
          egui_renderer.callback_resources.insert(taiko_renderer);
        }

        ui.painter().add(callback);

        let (id, mut rect) = ui.allocate_space(egui::vec2(ui.available_width(), 12.0));
        let response = ui.interact(rect, id, egui::Sense::drag());

        if response.hovered() {
          ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        rect = rect.shrink(2.0);

        rect.set_top(
          rect.top()
            + ui.ctx().animate_value_with_time(
              id.with("expand_anim"),
              if response.hovered() { 0.0 } else { 4.0 },
              0.05,
            ),
        );

        if response.drag_started() {
          self.last_state = clock.is_playing();

          if self.last_state {
            clock.set_playing(false);
          }
        }

        if let Some(pos) = response.interact_pointer_pos() {
          let pos = pos.x - rect.left();
          if pos > 0.0 && pos < rect.width() {
            let value = pos / rect.width();
            let time = value * clock.length().to_seconds() as f32;

            let bits = time.to_bits();
            if self.last_bits != bits {
              clock.set_position(Time::from_seconds(time));
              self.last_bits = bits;
            }
          }
        }

        if response.drag_stopped() {
          clock.set_playing(self.last_state);
        }

        let rounding = egui::Rounding::same(8.0);
        let inactive_color = ui.style().visuals.window_stroke.color;
        let active_color = egui::Color32::from_gray(120);

        ui.painter().rect(rect, rounding, inactive_color, egui::Stroke::NONE);

        let value = (time.to_seconds() / clock.length().to_seconds()).min(1.0);
        rect.set_width((rect.width() as f64 * value) as f32);

        ui.painter().rect(rect, rounding, active_color, egui::Stroke::NONE);
      });
  }

  pub fn change_beatmap(&mut self, graphics: &Graphics, egui_renderer: &mut egui_wgpu::Renderer, beatmap: &Beatmap) {
    let resources = egui_renderer.callback_resources.entry().or_insert_with(|| self.new_renderer.take().unwrap());

    self.current_beatmap = Some(beatmap.clone());
    resources.load_beatmap(&graphics.device, beatmap.clone());
    resources.set_hit_all(&graphics.queue);
  }

  pub fn scale(&mut self, scale_factor: f64) {
    self.scale_factor = scale_factor;
    self.new_scale_factor = Some(scale_factor);
  }
}

impl Drawable for BeatmapPreview {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    let hit_pos = PREVIEW_HEIGHT as f32 / 2.0;

    let mut renderer = TaikoRenderer::new(
      device,
      queue,
      format,
      TaikoRendererConfig {
        width: self.prev_width,
        height: PREVIEW_HEIGHT,
        scale_factor: self.scale_factor,
        scale: 0.425,
        zoom: self.zoom,
        hit_position_x: hit_pos / self.scale_factor as f32,
        hit_position_y: hit_pos / self.scale_factor as f32,
        don: self.don_color,
        kat: self.kat_color,
        hit_height: 12.5,
      },
    );

    let beatmap = self.current_beatmap.as_ref().unwrap().clone();
    renderer.load_beatmap(device, beatmap);
    renderer.set_hit_all(queue);

    self.new_renderer = Some(renderer);
  }
}
