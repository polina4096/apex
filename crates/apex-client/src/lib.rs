#![feature(map_many_mut)]
#![feature(adt_const_params)]
#![allow(incomplete_features)]

use apex_framework::{app::ApexFrameworkApplication, event::CoreEvent};
use client::{client::Client, event::ClientEvent};

use tap::Tap;
use winit::event_loop::{ControlFlow, EventLoop};

pub mod client;
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
  let mut app = ApexFrameworkApplication::<Client>::new(event_loop.create_proxy());

  event_loop.run_app(&mut app)?;

  // Unfortunate workaround for the fact that the window is unsound
  std::process::exit(0);
}
