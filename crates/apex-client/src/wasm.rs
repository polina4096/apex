#[cfg(target_arch="wasm32")]
pub mod wasm {
  use log::info;
use wasm_bindgen::prelude::*;

  use winit::{event_loop::{ControlFlow, EventLoopBuilder}, window::WindowBuilder};

  use crate::{client::event::ClientEvent, core::event::CoreEvent};

  #[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
  pub fn wasm_main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");

    let (event_loop, window) = crate::setup();

    use winit::platform::web::WindowExtWebSys;

    let web_window = web_sys::window().expect("Failed to get web window object");

    let size = {
      winit::dpi::LogicalSize::new(
        web_window.inner_width().unwrap().as_f64().unwrap(),
        web_window.inner_height().unwrap().as_f64().unwrap(),
      )
    };

    let window_ptr = std::ptr::addr_of!(window);
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_: web_sys::Event| {
      let web_window = web_sys::window().expect("Failed to get web window object");
      let size = winit::dpi::LogicalSize::new(
          web_window.inner_width().unwrap().as_f64().unwrap(),
          web_window.inner_height().unwrap().as_f64().unwrap(),
      );

      unsafe {
        let _ = window_ptr.as_ref().unwrap().request_inner_size(size);
      }
    }) as Box<dyn FnMut(_)>);

    web_window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref()).unwrap();
    closure.forget(); // Here we leak memory, but it's ok since this closure should have 'static anyways

    let document = web_window.document().expect("Failed to get the document");
    let dst = document.get_element_by_id("root").expect("Failed to find the canvas parent");
    let canvas = window.canvas().expect("Failed to create canvas element");
    dst.append_child(&canvas).expect("Failed to append canvas to the parent");
    let _ = window.request_inner_size(size);

    crate::run(event_loop, window).expect("Failed to run client");
  }
}
