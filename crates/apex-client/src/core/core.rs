use pollster::FutureExt;
use winit::{dpi::PhysicalSize, event_loop::{EventLoop, EventLoopProxy}, window::Window};

use super::{app::App, event::CoreEvent, graphics::{egui::EguiContext, graphics::Graphics}};

pub struct Core<'a, A: App> {
  pub proxy    : EventLoopProxy<CoreEvent<A::Event>>,

  pub window   : &'a Window,
  pub graphics : Graphics,
  pub egui_ctx : EguiContext,
}

impl<'a, A: App> Core<'a, A> {
  pub fn new(event_loop: &EventLoop<CoreEvent<A::Event>>, window: &'a Window) -> Self {
    let proxy = event_loop.create_proxy();
    let graphics = Graphics::new(window).block_on();
    let egui_ctx = EguiContext::new(event_loop, &graphics);

    return Self {
      proxy,
      window,
      graphics,
      egui_ctx,
    };
  }

  pub fn render(&mut self, app: &mut A) -> Result<(), wgpu::SurfaceError> {
    let output = self.graphics.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = self.graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("render encoder"),
    });

    {
      app.prepare(self, &mut encoder);

      let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("egui render pass"),
        color_attachments: &[
          Some(wgpu::RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: wgpu::Operations {
              load: wgpu::LoadOp::Load,
              store: wgpu::StoreOp::Store,
            },
          })
        ],
        timestamp_writes         : None,
        occlusion_query_set      : None,
        depth_stencil_attachment : None,
      });

      app.render(self, &mut rpass);
    }

    // submit work
    self.graphics.queue.submit(std::iter::once(encoder.finish()));
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

  pub fn exit(&self) {
    self.proxy.send_event(CoreEvent::Exit).unwrap();
  }
}
