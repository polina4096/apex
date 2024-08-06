use pollster::FutureExt;
use tap::Tap as _;
use triomphe::Arc;
use winit::{
  dpi::PhysicalSize,
  event_loop::{ActiveEventLoop, EventLoopProxy},
  window::Window,
};

use crate::client::{graphics::RenderingBackend, settings::Settings};

use super::{
  app::App,
  event::CoreEvent,
  graphics::{egui::Egui, graphics::Graphics},
};

pub struct Core<A: App> {
  pub proxy: EventLoopProxy<CoreEvent<A::Event>>,

  pub window: Arc<Window>,
  pub graphics: Graphics,
  pub egui: Egui,
}

impl<A: App> Core<A> {
  pub fn new(
    event_loop: &ActiveEventLoop,
    proxy: EventLoopProxy<CoreEvent<A::Event>>,
    window: Arc<Window>,
    settings: &Settings,
  ) -> Self {
    #[allow(clippy::infallible_destructuring_match)]
    let backend = match settings.graphics.rendering_backend() {
      RenderingBackend::Wgpu(wgpu_backend) => wgpu_backend,
    };

    let graphics = Graphics::new(
      &window,
      backend.into(),
      settings.graphics.present_mode().into(),
      settings.graphics.max_frame_latency(),
    )
    .block_on();

    let egui = Egui::new(event_loop, &graphics).tap(|egui| {
      egui.ctx().tap_deref(|ctx| {
        ctx.set_visuals(egui::Visuals::dark().tap_mut(|visuals| {
          visuals.window_highlight_topmost = false;
        }));

        ctx.options_mut(|options| {
          options.zoom_with_keyboard = false;
        });

        ctx.style_mut(|style| {
          style.interaction.selectable_labels = false;
        });
      });
    });

    return Self { proxy, window, graphics, egui };
  }

  pub fn render(&mut self, app: &mut A, settings: &mut Settings) -> Result<(), wgpu::SurfaceError> {
    let output = self.graphics.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let cmd_encoder_desc = wgpu::CommandEncoderDescriptor { label: Some("main command encoder") };
    let mut encoder = self.graphics.device.create_command_encoder(&cmd_encoder_desc);

    {
      app.prepare(self, settings, &mut encoder);

      let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("main render pass"),
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

      self.egui.resize(new_size);

      app.resize(self, new_size);
    }
  }

  pub fn scale(&mut self, app: &mut A, scale_factor: f64) {
    self.graphics.scale = scale_factor;
    self.egui.scale(scale_factor);

    app.scale(self, scale_factor);
  }

  pub fn recreate_context(&mut self, settings: &Settings) {
    let present_mode = settings.graphics.present_mode();
    let backend = settings.graphics.rendering_backend();
    let max_frame_latency = settings.graphics.max_frame_latency();

    #[allow(clippy::infallible_destructuring_match)]
    let backend = match backend {
      RenderingBackend::Wgpu(wgpu_backend) => wgpu_backend,
    };

    self.graphics = Graphics::new(&self.window, backend.into(), present_mode.into(), max_frame_latency).block_on();
    self.egui.recreate_context(&*self.window, &self.graphics);
  }

  pub fn exit(&self) {
    self.proxy.send_event(CoreEvent::Exit).unwrap();
  }
}
