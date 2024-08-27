use std::sync::atomic::AtomicBool;

use tap::Tap as _;
use triomphe::Arc;
use winit::{
  dpi::{LogicalSize, PhysicalSize},
  event_loop::{ActiveEventLoop, EventLoopProxy},
  window::Window,
};

use crate::{
  event::EventBus,
  graphics::presentation::{frame_limiter::FrameLimiter, frame_sync::FrameSync},
};

use super::{
  app::App,
  event::CoreEvent,
  graphics::{egui::Egui, graphics::Graphics},
};

pub struct Core<A: App> {
  pub event_bus: EventBus<A::Event>,
  pub window: Arc<Window>,
  pub frame_limiter: FrameLimiter,
  pub frame_sync: FrameSync,
  pub graphics: Graphics,
  pub egui: Egui,
}

impl<A: App> Core<A> {
  pub fn new(
    event_loop: &ActiveEventLoop,
    proxy: EventLoopProxy<CoreEvent<A::Event>>,
    window: Arc<Window>,
    app_focus: Arc<AtomicBool>,
    graphics: Graphics,
  ) -> Self {
    let event_bus = EventBus::new(proxy);

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

    let frame_limiter = FrameLimiter::new(Some(nz::u16!(60)), app_focus.clone());
    let frame_sync = FrameSync::new(app_focus.clone());

    return Self {
      event_bus,
      window,
      frame_limiter,
      frame_sync,
      graphics,
      egui,
    };
  }

  pub fn render(&mut self, app: &mut A) -> Result<(), wgpu::SurfaceError> {
    let output = self.graphics.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
    let cmd_encoder_desc = wgpu::CommandEncoderDescriptor { label: Some("main command encoder") };
    let mut encoder = self.graphics.device.create_command_encoder(&cmd_encoder_desc);

    app.prepare(self, &mut encoder);
    app.render(self, &mut encoder, view);

    // submit work
    self.graphics.queue.submit(std::iter::once(encoder.finish()));
    self.window.pre_present_notify();
    output.present();

    return Ok(());
  }

  pub fn resize(&mut self, app: &mut A, new_size: PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
      self.graphics.config.width = new_size.width;
      self.graphics.config.height = new_size.height;
      self.graphics.surface.configure(&self.graphics.device, &self.graphics.config);
      self.egui.resize(new_size);

      let LogicalSize { width, height } = new_size.to_logical(self.graphics.scale_factor as f64);
      self.graphics.width = width;
      self.graphics.height = height;
      app.resize(&self.graphics.device, &self.graphics.queue, width, height);
    }
  }

  pub fn scale(&mut self, app: &mut A, scale_factor: f32) {
    self.graphics.scale_factor = scale_factor;
    self.egui.scale(scale_factor);

    app.rescale(&self.graphics.device, &self.graphics.queue, scale_factor);
  }

  pub fn recreate_context(&mut self, graphics: Graphics) {
    self.graphics = graphics;
    self.egui.recreate_context(&*self.window, &self.graphics);
  }

  pub fn exit(&self) {
    self.event_bus.dispatch(CoreEvent::Exit);
  }
}
