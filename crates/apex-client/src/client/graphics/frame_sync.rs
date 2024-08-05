use std::sync::atomic::AtomicBool;

use triomphe::Arc;
use winit::window::Window;

pub struct FrameSync {
  #[allow(dead_code)]
  app_focus: Arc<AtomicBool>,

  #[cfg(target_os = "macos")]
  display_link: Option<display_link::DisplayLink>,

  current_window: Option<Arc<Window>>,
}

impl FrameSync {
  pub fn new(app_focus: Arc<AtomicBool>) -> Self {
    return Self {
      app_focus,

      #[cfg(target_os = "macos")]
      display_link: None,

      current_window: None,
    };
  }

  pub fn set_current_window(&mut self, window: Arc<Window>) {
    self.current_window = Some(window);
  }

  #[allow(clippy::result_unit_err)]
  pub fn enable_external_sync(&mut self, macos_fix: bool) -> Result<(), ()> {
    _ = macos_fix;

    #[cfg(target_os = "macos")]
    {
      use std::sync::atomic::Ordering;

      // Setup CVDisplayLink
      let Some(window) = self.current_window.clone() else {
        return Err(());
      };

      let app_focus = self.app_focus.clone();

      // This will be called on every vsync.
      let mut display_link = display_link::DisplayLink::new(move |_ts| {
        if macos_fix {
          // Make sure to request redraws only when the window is visible to prevent massive stutters :D
          if app_focus.load(Ordering::Relaxed) && window.is_visible().unwrap_or(false) {
            window.request_redraw();
          }
        } else {
          window.request_redraw();
        }
      })
      .unwrap();

      // Start the CVDisplayLink
      display_link.resume().unwrap();

      // CVDisplayLink must live as long it's used, otherwise nothing will happen.
      self.display_link = Some(display_link);
    }

    return Ok(());
  }

  pub fn disable_external_sync(&mut self) {
    #[cfg(target_os = "macos")]
    {
      // Stop the CVDisplayLink
      if let Some(mut display_link) = self.display_link.take() {
        display_link.pause().unwrap();
      }
    }
  }
}
