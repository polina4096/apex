#![feature(map_many_mut)]

use core::{
  core::Core,
  event::{CoreEvent, EventBus},
  graphics::{drawable::Drawable as _, egui::EguiContext, graphics::Graphics},
};
use std::sync::Arc;

use client::{
  client::Client,
  event::ClientEvent,
  state::{
    graphics_state::{FrameLimiterOptions, RenderingBackend},
    AppState,
  },
  util::frame_limiter::FrameLimiter,
};
use log::warn;
use pollster::FutureExt as _;
use wgpu::rwh::HasDisplayHandle;
use winit::{
  dpi::LogicalSize,
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
  window::{Window, WindowBuilder},
};

pub mod client;
pub mod core;
pub mod wasm;

pub fn setup() -> (EventLoop<CoreEvent<ClientEvent>>, Window) {
  let event_loop = EventLoopBuilder::<CoreEvent<ClientEvent>>::with_user_event()
    .build()
    .expect("Failed to create event loop");

  let window = WindowBuilder::new()
    .with_inner_size(LogicalSize::new(1200.0, 800.0))
    .build(&event_loop)
    .expect("Failed to create window");

  event_loop.set_control_flow(ControlFlow::Poll);

  return (event_loop, window);
}

pub fn run(event_loop: EventLoop<CoreEvent<ClientEvent>>, window: Window) -> color_eyre::Result<()> {
  let window = Arc::new(window);
  let app_state = AppState::default();
  let mut core = Core::new(&event_loop, &window, &app_state);
  let mut client = Client::new(&mut core, app_state, EventBus::new(event_loop.create_proxy()));

  let external_sync = client.app_state.graphics.frame_limiter == FrameLimiterOptions::DisplayLink;
  let is_unlimited = client.app_state.graphics.frame_limiter == FrameLimiterOptions::Unlimited;
  let target_fps = match client.app_state.graphics.frame_limiter {
    FrameLimiterOptions::Custom(fps) => fps as u16,
    _ => 120,
  };

  let mut frame_limiter = FrameLimiter::new(external_sync, is_unlimited, target_fps);

  window.request_redraw();

  event_loop.run(|event, elwt| {
    match event {
      Event::UserEvent(event) => {
        match event {
          CoreEvent::Exit => {
            elwt.exit();
          }

          CoreEvent::RecreateGraphicsContext => {
            let present_mode = client.app_state.graphics.present_mode;
            let backend = client.app_state.graphics.rendering_backend;

            #[rustfmt::skip]
            let RenderingBackend::Wgpu(backend) = backend else { todo!() };

            core.graphics = Graphics::new(core.window, backend.into(), present_mode.into()).block_on();

            let display_handle = elwt.display_handle().unwrap();
            core.egui_ctx = EguiContext::new(&display_handle, &core.graphics);
            client.recreate(&core.graphics);
          }

          CoreEvent::ReconfigureSurfaceTexture => {
            core.graphics.config.present_mode = client.app_state.graphics.present_mode.into();
            core.graphics.surface.configure(&core.graphics.device, &core.graphics.config);
          }

          CoreEvent::UpdateFrameLimiterConfiguration => {
            match client.app_state.graphics.frame_limiter {
              FrameLimiterOptions::Custom(fps) => {
                frame_limiter.disable_external_sync();
                frame_limiter.set_unlimited(false);
                frame_limiter.set_target_fps(fps as u16);
              }

              FrameLimiterOptions::DisplayLink => {
                frame_limiter.enable_external_sync(window.clone());
              }

              FrameLimiterOptions::Unlimited => {
                frame_limiter.disable_external_sync();
                frame_limiter.set_unlimited(true);
              }
            }
          }

          CoreEvent::User(event) => {
            client.dispatch(&mut core, event);
          }
        }
      }

      Event::AboutToWait => {
        frame_limiter.request_redraw(&window);
      }

      Event::WindowEvent { event, .. } => {
        let winit_state = &mut core.egui_ctx.winit_state;
        let result = winit_state.on_window_event(&window, &event);
        #[rustfmt::skip] if result.consumed { return };

        match event {
          WindowEvent::CloseRequested => {
            elwt.exit();
          }

          WindowEvent::Focused(focused) => {
            frame_limiter.update_focus(focused);
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

          WindowEvent::DroppedFile(path) => {
            match std::fs::read(&path) {
              Ok(file) => client.file(&mut core, path, file),
              Err(err) => warn!("Failed to read dropped file: {:?}", err),
            };
          }

          _ => {}
        }
      }

      _ => {}
    }
  })?;

  return Ok(());
}
