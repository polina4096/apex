use egui_winit::winit::dpi::PhysicalSize;

use crate::{graphics::context::Graphics, binds::KeyCombination};

#[allow(unused_variables)]
pub trait Layer<'b, State: 'b> {
    fn draw<'a: 'b>(&'a mut self, state: State, render_pass: &mut wgpu::RenderPass<'b>, graphics: &mut Graphics) { }
    fn action(&mut self, keys: KeyCombination, state: State) -> bool;
    fn resize(&mut self, new_size: PhysicalSize<u32>) { }
    fn scale(&mut self, scale: f64) { }
}