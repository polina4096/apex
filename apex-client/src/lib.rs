#![allow(clippy::needless_return)]
#![allow(clippy::redundant_field_names)]
#![allow(clippy::extra_unused_type_parameters)]
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::new_without_default)]
#![allow(clippy::collapsible_match)]
#![allow(clippy::iter_nth_zero)]
#![feature(min_specialization)]
#![feature(exitcode_exit_method)]
#![feature(let_chains)]
use clap::Parser;
use log::{warn};
use winit::{event_loop::{ControlFlow, EventLoopBuilder}, window::{WindowBuilder}, event::{Event, WindowEvent}};
#[cfg(not(target_arch = "wasm32"))] use winit::dpi::LogicalSize;

#[cfg(target_arch = "wasm32")] use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")] use winit::window::Window;

use crate::{config::Config, app::App};

pub mod app;
pub mod state;
pub mod graphics;
pub mod config;
pub mod view;
pub mod taiko;
pub mod layer;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub async fn run() {
    let config = Config::parse();
    
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");

            let web_window = web_sys::window().expect("Failed to get web window object");
            let size = {
                winit::dpi::LogicalSize::new( // TODO: unwraps
                    web_window.inner_width().unwrap().as_f64().unwrap(),
                    web_window.inner_height().unwrap().as_f64().unwrap(),
                )
            };
        } else {
            pretty_env_logger::init();
            
            let size = LogicalSize::new(1000, 625);
        }
    }

    let event_loop = EventLoopBuilder::<()>::with_user_event().build();
    let window = WindowBuilder::new()
        .with_title("Apex")
        .with_inner_size(size)
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")] {
        let window_ptr = std::ptr::addr_of!(window);
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
            let web_window = web_sys::window().expect("Failed to get web window object");
            let size = winit::dpi::LogicalSize::new(
                web_window.inner_width().unwrap().as_f64().unwrap(),
                web_window.inner_height().unwrap().as_f64().unwrap(),
            );

            unsafe { window_ptr.as_ref().unwrap().set_inner_size(size) }
        }) as Box<dyn FnMut(_)>);

        web_window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget(); // Here we leak memory, but it's ok since this closure should have 'static anyways
    }

    #[cfg(target_arch = "wasm32")] {
        use winit::platform::web::WindowExtWebSys;
        let document = web_window.document().expect("Failed to get the document");
        let dst = document.get_element_by_id("apex").expect("Failed to find the canvas parent");
        let canvas = web_sys::Element::from(window.canvas());
        dst.append_child(&canvas).expect("Failed to append canvas to the parent");
    }

    let proxy = event_loop.create_proxy();
    let mut app = App::new(window, &event_loop, proxy, &config).await;

    event_loop.run(move |event, _, control_flow| {
        app.update();

        match event {
            Event::RedrawRequested(window_id) if window_id == app.get_window().id() => {
                match app.render() {
                    Ok(_) => { }
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => app.resize(app.graphics.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => warn!("{:?}", e),
                }
            }

            Event::MainEventsCleared => {
                app.get_window().request_redraw();
            }

            Event::WindowEvent { event, window_id } if window_id == app.get_window().id() => {
                if !app.input(&event) {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit
                        }

                        WindowEvent::Resized(physical_size) => {
                            app.resize(physical_size);
                        }

                        WindowEvent::ScaleFactorChanged { new_inner_size, scale_factor } => {
                            app.resize(*new_inner_size);
                            app.scale(scale_factor);
                        }

                        _ => {}
                    }
                }
            }

            Event::UserEvent(_event) => {

            }
            
            _ => {}
        }
    });
}
