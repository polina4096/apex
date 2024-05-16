use std::fmt::Debug;

use super::core::Core;

pub trait App: Sized {
  type Event: Debug + 'static;

  fn prepare(&mut self, core: &mut Core<Self>, encoder: &mut wgpu::CommandEncoder);
  fn render<'rpass>(&'rpass self, core: &'rpass mut Core<Self>, rpass: &mut wgpu::RenderPass<'rpass>);

  #[allow(unused_variables)]
  fn resize(&mut self, core: &mut Core<Self>, size: winit::dpi::PhysicalSize<u32>) {}

  #[allow(unused_variables)]
  fn scale(&mut self, core: &mut Core<Self>, scale_factor: f64) {}
}
