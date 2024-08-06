use std::sync::atomic::{AtomicBool, Ordering};

use triomphe::Arc;
use winit::{
  application::ApplicationHandler,
  dpi::LogicalSize,
  event::WindowEvent,
  event_loop::{ActiveEventLoop, EventLoopProxy},
  window::{Window, WindowId},
};

use crate::{
  client::{
    client::Client,
    event::ClientEvent,
    graphics::{frame_limiter::FrameLimiter, frame_sync::FrameSync, FrameLimiterOptions},
    settings::Settings,
  },
  core::{
    core::Core,
    data::persistent::Persistent as _,
    event::{CoreEvent, EventBus},
    graphics::drawable::Drawable as _,
  },
};

pub struct ApexApp {
  proxy: EventLoopProxy<CoreEvent<ClientEvent>>,

  settings: Settings,
  client: Option<Client>,
  core: Option<Core<Client>>,

  app_focus: Arc<AtomicBool>,
  frame_limiter: FrameLimiter,
  frame_sync: FrameSync,
}

impl ApexApp {
  pub fn new(proxy: EventLoopProxy<CoreEvent<ClientEvent>>) -> Self {
    let app_focus = Arc::new(AtomicBool::new(true));

    let settings = Settings::load("./config.toml");

    // Frame limiter setup
    let is_unlimited = settings.graphics.frame_limiter() == FrameLimiterOptions::Unlimited;
    let target_fps = match settings.graphics.frame_limiter() {
      FrameLimiterOptions::Custom(fps) => fps as u16,
      _ => 120,
    };

    let frame_limiter = FrameLimiter::new(is_unlimited, target_fps, app_focus.clone());
    let frame_sync = FrameSync::new(app_focus.clone());

    return Self {
      proxy,

      settings,
      client: None,
      core: None,

      app_focus,
      frame_limiter,
      frame_sync,
    };
  }
}

impl ApplicationHandler<CoreEvent<ClientEvent>> for ApexApp {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let window_attrs = Window::default_attributes() //
      .with_inner_size(LogicalSize::new(1200, 800));

    let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

    // Setup external frame synchronization
    self.frame_sync.set_current_window(window.clone());

    if self.settings.graphics.frame_limiter() == FrameLimiterOptions::DisplayLink {
      self.frame_sync.enable_external_sync(self.settings.graphics.macos_stutter_fix()).unwrap();
    }

    // Workaround for the first frame not being rendered on some platforms
    window.request_redraw();

    let mut core = Core::new(event_loop, self.proxy.clone(), window.clone(), &self.settings);
    let client = Client::new(&mut core, &self.settings, EventBus::new(self.proxy.clone()));

    self.client = Some(client);
    self.core = Some(core);
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    let Some(core) = &self.core else { return };

    let external_sync = self.settings.graphics.frame_limiter() == FrameLimiterOptions::DisplayLink;

    if !external_sync {
      self.frame_limiter.request_redraw(&core.window);
    }
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
        match core.render(client, &mut self.settings) {
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
          Ok(file) => client.file(core, path, file),
          Err(err) => log::warn!("Failed to read dropped file: {:?}", err),
        };
      }

      _ => {}
    }
  }

  fn user_event(&mut self, event_loop: &ActiveEventLoop, event: CoreEvent<ClientEvent>) {
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
        core.recreate_context(&self.settings);
        client.recreate(&core.graphics.device, &core.graphics.queue, core.graphics.config.format);
      }

      CoreEvent::UpdateFrameLimiterConfiguration => {
        match self.settings.graphics.frame_limiter() {
          FrameLimiterOptions::Custom(fps) => {
            self.frame_sync.disable_external_sync();

            self.frame_limiter.set_unlimited(false);
            self.frame_limiter.set_target_fps(fps as u16);
          }

          FrameLimiterOptions::DisplayLink => {
            self.frame_sync.enable_external_sync(self.settings.graphics.macos_stutter_fix()).unwrap();
          }

          FrameLimiterOptions::Unlimited => {
            self.frame_sync.disable_external_sync();

            self.frame_limiter.set_unlimited(true);
          }
        }
      }

      CoreEvent::User(event) => {
        client.dispatch(core, event);
      }
    }
  }

  fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
    self.settings.save("./config.toml");
    self.client.as_ref().unwrap().input.keybinds.save("./keybinds.toml");
  }
}
