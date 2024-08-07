use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use triomphe::Arc;
use winit::event::{KeyEvent, Modifiers};
use winit::{
  application::ApplicationHandler,
  dpi::LogicalSize,
  event::WindowEvent,
  event_loop::{ActiveEventLoop, EventLoopProxy},
  window::{Window, WindowId},
};

use crate::graphics::graphics::Graphics;
use crate::{core::Core, event::CoreEvent, graphics::drawable::Drawable};

#[allow(unused_variables)]
pub trait App: Drawable + Sized {
  type Event: Debug + 'static;

  fn create(
    event_loop: &ActiveEventLoop,
    window: Arc<Window>,
    app_focus: Arc<AtomicBool>,
    proxy: EventLoopProxy<CoreEvent<Self::Event>>,
  ) -> (Self, Core<Self>);

  fn recreate_graphics(&mut self, core: &mut Core<Self>) -> Graphics;

  fn destroy(&self) {}

  fn prepare(&mut self, core: &mut Core<Self>, encoder: &mut wgpu::CommandEncoder) {}
  fn render<'rpass>(&'rpass self, core: &'rpass mut Core<Self>, rpass: &mut wgpu::RenderPass<'rpass>) {}
  fn resize(&mut self, core: &mut Core<Self>, size: winit::dpi::PhysicalSize<u32>) {}
  fn scale(&mut self, core: &mut Core<Self>, scale_factor: f64) {}
  fn input(&mut self, core: &mut Core<Self>, event: KeyEvent) {}
  fn modifiers(&mut self, modifiers: Modifiers) {}
  fn dispatch(&mut self, core: &mut Core<Self>, event: Self::Event) {}
  fn file_dropped(&mut self, _core: &mut Core<Self>, path: PathBuf, file: Vec<u8>) {}
}

pub struct ApexFrameworkApplication<A: App> {
  proxy: EventLoopProxy<CoreEvent<A::Event>>,

  client: Option<A>,
  core: Option<Core<A>>,

  app_focus: Arc<AtomicBool>,
}

impl<A: App> ApexFrameworkApplication<A> {
  pub fn new(proxy: EventLoopProxy<CoreEvent<A::Event>>) -> Self {
    let app_focus = Arc::new(AtomicBool::new(true));

    return Self {
      proxy,

      client: None,
      core: None,

      app_focus,
    };
  }
}

impl<A: App> ApplicationHandler<CoreEvent<A::Event>> for ApexFrameworkApplication<A> {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attrs = Window::default_attributes() //
      .with_inner_size(LogicalSize::new(1200, 800));

    let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

    // Workaround for the first frame not being rendered on some platforms
    window.request_redraw();

    // Initialize the client and core
    let (client, core) = A::create(event_loop, window.clone(), self.app_focus.clone(), self.proxy.clone());

    self.client = Some(client);
    self.core = Some(core);
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    let Some(core) = &mut self.core else { return };
    core.frame_limiter.request_redraw(&core.window);
  }

  fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
    let Some(core) = &mut self.core else { return };
    let Some(client) = &mut self.client else { return };

    let is_context_open = core.egui.ctx().is_context_menu_open();
    // TODO: this might not be the best way to capture (disable) unwanted scrolling
    if !(is_context_open && matches!(event, WindowEvent::MouseWheel { .. })) {
      let result = core.egui.handle_window_event(&core.window, &event);
      #[rustfmt::skip] if result.consumed { return };
    }

    match event {
      WindowEvent::CloseRequested => {
        event_loop.exit();
      }

      WindowEvent::Focused(focused) => {
        self.app_focus.store(focused, Ordering::Relaxed);
      }

      WindowEvent::KeyboardInput { event, .. } => {
        if core.egui.ctx().is_context_menu_open() {
          return;
        }

        client.input(core, event);
      }

      WindowEvent::ModifiersChanged(modifiers) => {
        client.modifiers(modifiers);
      }

      WindowEvent::Resized(size) => {
        core.resize(client, size);
      }

      WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
        core.scale(client, scale_factor);
      }

      WindowEvent::RedrawRequested => {
        match core.render(client) {
          Ok(_) => {}

          // Reconfigure the surface if lost
          Err(wgpu::SurfaceError::Lost) => core.resize(client, core.graphics.size),

          // The system is out of memory, we should probably quit
          Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),

          // All other errors (Outdated, Timeout) should be resolved by the next frame
          Err(e) => log::warn!("{:?}", e),
        }
      }

      WindowEvent::DroppedFile(path) => {
        match std::fs::read(&path) {
          Ok(file) => client.file_dropped(core, path, file),
          Err(err) => log::warn!("Failed to read dropped file: {:?}", err),
        };
      }

      _ => {}
    }
  }

  fn user_event(&mut self, event_loop: &ActiveEventLoop, event: CoreEvent<A::Event>) {
    let Some(core) = &mut self.core else { return };
    let Some(client) = &mut self.client else { return };

    match event {
      CoreEvent::Exit => {
        event_loop.exit();
      }

      CoreEvent::ReconfigureSurface => {
        core.graphics.surface.configure(&core.graphics.device, &core.graphics.config);
      }

      CoreEvent::RecreateGraphicsContext => {
        let graphics = client.recreate_graphics(core);

        core.recreate_context(graphics);
        client.recreate(&core.graphics.device, &core.graphics.queue, core.graphics.config.format);
      }

      CoreEvent::User(event) => {
        client.dispatch(core, event);
      }
    }
  }

  fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
    let Some(client) = &self.client else { return };
    client.destroy();
  }
}
