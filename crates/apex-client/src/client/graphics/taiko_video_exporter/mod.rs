use apex_framework::{graphics::video_exporter::VideoExporterCallback, time::time::Time};

use super::taiko_renderer::taiko_renderer::TaikoRenderer;

pub struct TaikoVideoExporterCallback<'a> {
  taiko_renderer: &'a mut TaikoRenderer,

  preview_time: u64,
}

impl<'a> TaikoVideoExporterCallback<'a> {
  pub fn new(taiko_renderer: &'a mut TaikoRenderer, preview_time: u64) -> Self {
    return Self { taiko_renderer, preview_time };
  }
}

impl<'a> VideoExporterCallback for TaikoVideoExporterCallback<'a> {
  type Data = i32;

  fn prepare_frame(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, i: Self::Data) {
    let time = Time::from_ms(i as f64 / 120.0 * 1000.0 + self.preview_time as f64);
    self.taiko_renderer.prepare(queue, time);
  }

  fn render_frame<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
    self.taiko_renderer.render(rpass);
  }
}
