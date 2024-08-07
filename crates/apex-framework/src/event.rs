use std::fmt::Debug;

use winit::event_loop::EventLoopProxy;

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
    self.proxy.send_event(CoreEvent::User(event)).unwrap();
  }
}
