use std::sync::atomic::{AtomicBool, Ordering};

use instant::Instant;
use triomphe::Arc;
use winit::window::Window;

pub struct FrameLimiter {
  app_focus: Arc<AtomicBool>,
  last_frame: Instant,

  is_enabled: bool,
  unlimited: bool,
  target_fps: u16,
}

impl FrameLimiter {
  pub fn new(unlimited: bool, target_fps: u16, app_focus: Arc<AtomicBool>) -> Self {
    return Self {
      app_focus,
      last_frame: Instant::now(),

      is_enabled: true,
      unlimited,
      target_fps,
    };
  }

  /// Requests a redraw if the frame limiter allows it.
  pub fn request_redraw(&mut self, window: &Window) {
    if !self.is_enabled {
      return;
    }

    if self.unlimited {
      window.request_redraw();
      return;
    }

    let mut fps = self.target_fps;
    if !self.app_focus.load(Ordering::Relaxed) || !window.is_visible().unwrap_or(false) {
      fps = 30; // fps when not focused
    }

    let now = Instant::now();
    if now.duration_since(self.last_frame).as_micros() >= (1000 * 1000) / fps as u128 {
      window.request_redraw();
      self.last_frame = now;
    }
  }

  /// Use this to set the desired target FPS.
  pub fn set_target_fps(&mut self, fps: u16) {
    self.target_fps = fps;
  }

  /// Controls whether the frame limiter is enabled or not.
  pub fn set_unlimited(&mut self, unlimited: bool) {
    self.unlimited = unlimited;
  }

  pub fn set_enabled(&mut self, enabled: bool) {
    self.is_enabled = enabled;
  }

  pub fn is_enabled(&self) -> bool {
    return self.is_enabled;
  }
}
