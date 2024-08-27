use std::fmt::Debug;

use winit::event_loop::{EventLoopClosed, EventLoopProxy};

#[derive(Debug, PartialEq, Eq)]
pub enum CoreEvent<T> {
  Exit,
  RecreateGraphicsContext,
  ReconfigureSurface,
  User(T),
}

#[derive(Debug)]
pub struct EventBus<T: 'static> {
  proxy: EventLoopProxy<CoreEvent<T>>,
}

impl<T: 'static> Clone for EventBus<T> {
  fn clone(&self) -> Self {
    return Self { proxy: self.proxy.clone() };
  }
}

impl<T: Debug> EventBus<T> {
  pub fn new(proxy: EventLoopProxy<CoreEvent<T>>) -> Self {
    return Self { proxy };
  }

  pub fn send(&self, event: T) {
    self.dispatch(CoreEvent::User(event));
  }

  pub fn dispatch(&self, event: CoreEvent<T>) {
    if let Err(EventLoopClosed(event)) = self.proxy.send_event(event) {
      log::error!("Failed to send {:?} through EventLoopProxy.", event);
    }
  }
}
