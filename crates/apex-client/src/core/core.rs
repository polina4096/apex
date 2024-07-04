use pollster::FutureExt;
use tap::Tap as _;
use winit::{
  dpi::PhysicalSize,
  event_loop::{EventLoop, EventLoopProxy},
  window::Window,
};

use crate::client::state::{graphics_state::RenderingBackend, AppState};

use super::{
  app::App,
  event::CoreEvent,
  graphics::{egui::EguiContext, graphics::Graphics},
};

pub struct Core<'a, A: App> {
  pub proxy: EventLoopProxy<CoreEvent<A::Event>>,

  pub window: &'a Window,
  pub graphics: Graphics,
  pub egui_ctx: EguiContext,
}

impl<'a, A: App> Core<'a, A> {
  pub fn new(event_loop: &EventLoop<CoreEvent<A::Event>>, window: &'a Window, app_state: &AppState) -> Self {
    let proxy = event_loop.create_proxy();

    #[rustfmt::skip]
    let RenderingBackend::Wgpu(backend) = app_state.graphics.rendering_backend else { todo!() };
    let graphics = Graphics::new(window, backend.into(), app_state.graphics.present_mode.into()).block_on();

    let egui_ctx = EguiContext::new(event_loop, &graphics);
    egui_ctx.egui_ctx().set_visuals(egui::Visuals::dark().tap_mut(|vis| {
      vis.window_highlight_topmost = false;
    }));

    return Self { proxy, window, graphics, egui_ctx };
  }

  pub fn render(&mut self, app: &mut A) -> Result<(), wgpu::SurfaceError> {
    let output = self.graphics.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let cmd_encoder_desc = wgpu::CommandEncoderDescriptor { label: Some("render encoder") };
    let mut encoder = self.graphics.device.create_command_encoder(&cmd_encoder_desc);

    {
      app.prepare(self, &mut encoder);

      let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("egui render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
          },
        })],
        timestamp_writes: None,
        occlusion_query_set: None,
        depth_stencil_attachment: None,
      });

      app.render(self, &mut rpass);
    }

    // submit work
    self.graphics.queue.submit(std::iter::once(encoder.finish()));
    self.window.pre_present_notify();
    output.present();

    return Ok(());
  }

  pub fn resize(&mut self, app: &mut A, new_size: PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.graphics.size = new_size;
      self.graphics.config.width = new_size.width;
      self.graphics.config.height = new_size.height;
      self.graphics.surface.configure(&self.graphics.device, &self.graphics.config);

      self.egui_ctx.resize(new_size);

      app.resize(self, new_size);
    }
  }

  pub fn scale(&mut self, app: &mut A, scale_factor: f64) {
    self.graphics.scale = scale_factor;
    self.egui_ctx.scale(scale_factor);

    app.scale(self, scale_factor);
  }

  pub fn egui_ctx(&self) -> &egui::Context {
    return self.egui_ctx.egui_ctx();
  }

  pub fn exit(&self) {
    self.proxy.send_event(CoreEvent::Exit).unwrap();
  }

  pub fn recreate_graphics_context(&self) {
    self.proxy.send_event(CoreEvent::RecreateGraphicsContext).unwrap();
  }

  pub fn reconfigure_surface_texture(&self) {
    self.proxy.send_event(CoreEvent::ReconfigureSurfaceTexture).unwrap();
  }

  pub fn update_frame_limiter_configuration(&self) {
    self.proxy.send_event(CoreEvent::UpdateFrameLimiterConfiguration).unwrap();
  }
}
