use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use triomphe::Arc;
use winit::dpi::PhysicalSize;
use winit::event::{KeyEvent, Modifiers};
use winit::window::WindowAttributes;
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
  fn destroy(&self) {}

  fn recreate_graphics(&mut self, core: &mut Core<Self>) -> Graphics;

  fn window_attrs() -> WindowAttributes;

  fn prepare(&mut self, core: &mut Core<Self>, encoder: &mut wgpu::CommandEncoder) {}
  fn render(&self, core: &mut Core<Self>, encoder: &mut wgpu::CommandEncoder, view: wgpu::TextureView) {}
  fn input(&mut self, core: &mut Core<Self>, event: KeyEvent) {}
  fn modifiers(&mut self, core: &mut Core<Self>, modifiers: Modifiers) {}
  fn dispatch(&mut self, core: &mut Core<Self>, event: Self::Event) {}
  fn file_dropped(&mut self, core: &mut Core<Self>, path: PathBuf, file: Vec<u8>) {}
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
    let window_attrs = A::window_attrs() //
      .with_inner_size(LogicalSize::new(1200, 800));

    let window = match event_loop.create_window(window_attrs) {
      Ok(x) => Arc::new(x),
      Err(e) => {
        log::error!("Failed to create window: {:?}", e);
        return;
      }
    };

    // Initialize the client and core
    let (client, core) = A::create(event_loop, window.clone(), self.app_focus.clone(), self.proxy.clone());

    self.client = Some(client);
    self.core = Some(core);

    // Workaround for the first frame not being rendered on some platforms
    window.request_redraw();
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

    // Feed the event to the egui context
    if !matches!(event, WindowEvent::MouseWheel { .. }) {
      _ = core.egui.handle_window_event(&core.window, &event);
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
        client.modifiers(core, modifiers);
      }

      WindowEvent::Resized(size) => {
        core.resize(client, size);
      }

      WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
        core.scale(client, scale_factor as f32);
      }

      WindowEvent::RedrawRequested => {
        match core.render(client) {
          Ok(_) => {}

          // Reconfigure the surface if lost
          Err(wgpu::SurfaceError::Lost) => {
            let wgpu::SurfaceConfiguration { width, height, .. } = core.graphics.config;
            core.resize(client, PhysicalSize::new(width, height));
          }

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
