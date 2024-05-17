#![feature(map_many_mut)]

use core::{core::Core, event::{CoreEvent, EventBus}};

use client::{client::Client, event::ClientEvent};
use instant::Instant;
use log::warn;
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
  window::{Window, WindowBuilder},
};

pub mod client;
pub mod core;
pub mod wasm;

pub fn setup() -> (EventLoop<CoreEvent<ClientEvent>>, Window) {
  let event_loop = EventLoopBuilder::<CoreEvent<ClientEvent>>::with_user_event().build().expect("Failed to create event loop");
  let window = WindowBuilder::new().build(&event_loop).expect("Failed to create window");
  event_loop.set_control_flow(ControlFlow::Poll);
  return (event_loop, window);
}

pub fn run(event_loop: EventLoop<CoreEvent<ClientEvent>>, window: Window) -> color_eyre::Result<()> {
  event_loop.set_control_flow(ControlFlow::Poll);

  let mut core = Core::new(&event_loop, &window);
  let mut client = Client::new(&mut core, EventBus::new(event_loop.create_proxy()));

  let mut app_focus = false;
  let mut last_frame = Instant::now();

  event_loop.run(|event, elwt| {
    if !app_focus || !window.is_visible().unwrap_or(false) {
      let now = Instant::now();
      if now.duration_since(last_frame).as_micros() >= (1000 * 1000) / 120 {
        window.request_redraw();
        last_frame = now;
      }
    } else {
      window.request_redraw();
    }

    match event {
      Event::UserEvent(event) => {
        match event {
          CoreEvent::Exit => {
            elwt.exit();
          }

          CoreEvent::User(event) => {
            client.dispatch(&mut core, event);
          }
        }
      }

      Event::WindowEvent { event, .. } => {
        if core.egui_ctx
          .winit_state
          .on_window_event(&window, &event)
          .consumed
        {
          return;
        }

        match event {
          WindowEvent::CloseRequested => {
            elwt.exit();
          }

          WindowEvent::Focused(focused) => {
            app_focus = focused;
          }

          WindowEvent::KeyboardInput { event, .. } => {
            client.input(&mut core, event);
          }

          WindowEvent::ModifiersChanged(modifiers) => {
            client.modifiers(modifiers);
          }

          WindowEvent::Resized(size) => {
            core.resize(&mut client, size);
          }

          WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            core.scale(&mut client, scale_factor);
          }

          WindowEvent::RedrawRequested => {
            match core.render(&mut client) {
              Ok(_) => {}

              // Reconfigure the surface if lost
              Err(wgpu::SurfaceError::Lost) => core.resize(&mut client, core.graphics.size),

              // The system is out of memory, we should probably quit
              Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),

              // All other errors (Outdated, Timeout) should be resolved by the next frame
              Err(e) => warn!("{:?}", e),
            }
          }

          _ => { }
        }

      }

      _ => { }
    }
  })?;

  return Ok(());
}
