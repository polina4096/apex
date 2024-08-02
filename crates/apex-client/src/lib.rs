#![feature(map_many_mut)]
#![feature(adt_const_params)]
#![allow(incomplete_features)]

use apex::Apex;
use client::event::ClientEvent;
use core::event::CoreEvent;

use tap::Tap;
use winit::event_loop::{ControlFlow, EventLoop};

pub mod apex;
pub mod client;
pub mod core;
pub mod wasm;

pub fn create_event_loop() -> EventLoop<CoreEvent<ClientEvent>> {
  return EventLoop::<CoreEvent<ClientEvent>>::with_user_event()
    .build()
    .expect("Failed to create event loop")
    .tap_mut(|el| {
      el.set_control_flow(ControlFlow::Poll);
    });
}

pub fn run(event_loop: EventLoop<CoreEvent<ClientEvent>>) -> color_eyre::Result<()> {
  let proxy = event_loop.create_proxy();
  let mut app = Apex::new(proxy);

  event_loop.run_app(&mut app)?;

  // Unfortunate workaround for the fact that the window is unsound
  std::process::exit(0);
}
