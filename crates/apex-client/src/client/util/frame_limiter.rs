use std::sync::Arc;

use instant::Instant;
use winit::window::Window;

pub struct FrameLimiter {
  app_focus: bool,
  last_frame: Instant,

  external_sync: bool,
  unlimited: bool,
  target_fps: u16,

  #[cfg(target_os = "macos")]
  display_link: Option<display_link::DisplayLink>,
  window: Arc<Window>,
}

impl FrameLimiter {
  pub fn new(window: Arc<Window>, external: bool, unlimited: bool, target_fps: u16) -> Self {
    let mut limiter = Self {
      app_focus: false,
      last_frame: Instant::now(),

      external_sync: external,
      unlimited,
      target_fps,

      #[cfg(target_os = "macos")]
      display_link: None,
      window,
    };

    if external {
      limiter.enable_external_sync();
    }

    return limiter;
  }

  /// Requests a redraw if the frame limiter allows it.
  pub fn request_redraw(&mut self, window: &Window) {
    if self.external_sync {
      return;
    }

    if self.unlimited {
      window.request_redraw();
      return;
    }

    let mut fps = self.target_fps;
    if !self.app_focus || !window.is_visible().unwrap_or(false) {
      fps = 30; // fps when not focused
    }

    let now = Instant::now();
    if now.duration_since(self.last_frame).as_micros() >= (1000 * 1000) / fps as u128 {
      window.request_redraw();
      self.last_frame = now;
    }
  }

  /// Call this when the app gains or loses focus.
  pub fn update_focus(&mut self, focus: bool) {
    self.app_focus = focus;
  }

  /// Use this to set the desired target FPS.
  pub fn set_target_fps(&mut self, fps: u16) {
    self.target_fps = fps;
  }

  /// Controls whether the frame limiter is enabled or not.
  pub fn set_unlimited(&mut self, unlimited: bool) {
    self.unlimited = unlimited;
  }

  pub fn enable_external_sync(&mut self) {
    self.external_sync = true;

    #[cfg(target_os = "macos")]
    {
      // Setup CVDisplayLink
      let window = self.window.clone();
      let mut display_link = display_link::DisplayLink::new(move |_ts| {
        // This will be called on every vsync.
        window.request_redraw();
      })
      .unwrap();

      // Start the CVDisplayLink
      display_link.resume().unwrap();

      // CVDisplayLink must live as long it's used, otherwise nothing will happen.
      self.display_link = Some(display_link);
    }
  }

  pub fn disable_external_sync(&mut self) {
    self.external_sync = false;

    #[cfg(target_os = "macos")]
    {
      // Stop the CVDisplayLink
      if let Some(mut display_link) = self.display_link.take() {
        display_link.pause().unwrap();
      }
    }
  }
}
