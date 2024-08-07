use std::{
  num::NonZero,
  sync::atomic::{AtomicBool, Ordering},
};

use instant::Instant;
use triomphe::Arc;
use winit::window::Window;

pub struct FrameLimiter {
  app_focus: Arc<AtomicBool>,
  last_frame: Instant,

  is_enabled: bool,
  target_fps: Option<NonZero<u16>>,
}

impl FrameLimiter {
  pub fn new(target_fps: Option<NonZero<u16>>, app_focus: Arc<AtomicBool>) -> Self {
    return Self {
      app_focus,
      last_frame: Instant::now(),

      is_enabled: true,
      target_fps,
    };
  }

  /// Requests a redraw if the frame limiter allows it.
  pub fn request_redraw(&mut self, window: &Window) {
    if !self.is_enabled {
      return;
    }

    if self.target_fps.is_none() {
      window.request_redraw();
      return;
    }

    if let Some(mut fps) = self.target_fps {
      if !self.app_focus.load(Ordering::Relaxed) || !window.is_visible().unwrap_or(false) {
        fps = NonZero::new(30).unwrap(); // fps when not focused
      }

      let now = Instant::now();
      if now.duration_since(self.last_frame).as_micros() >= (1000 * 1000) / fps.get() as u128 {
        window.request_redraw();
        self.last_frame = now;
      }
    }
  }

  /// Use this to set the desired target FPS. None means unlimited.
  pub fn set_target_fps(&mut self, fps: Option<NonZero<u16>>) {
    self.target_fps = fps;
  }

  pub fn set_enabled(&mut self, enabled: bool) {
    self.is_enabled = enabled;
  }

  pub fn is_enabled(&self) -> bool {
    return self.is_enabled;
  }
}
