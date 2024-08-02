use std::fmt::Debug;

use crate::client::settings::Settings;

use super::{core::Core, graphics::drawable::Drawable};

pub trait App: Drawable + Sized {
  type Event: Debug + 'static;

  fn prepare(&mut self, core: &mut Core<Self>, settings: &mut Settings, encoder: &mut wgpu::CommandEncoder);
  fn render<'rpass>(&'rpass self, core: &'rpass mut Core<Self>, rpass: &mut wgpu::RenderPass<'rpass>);

  #[allow(unused_variables)]
  fn resize(&mut self, core: &mut Core<Self>, size: winit::dpi::PhysicalSize<u32>) {}

  #[allow(unused_variables)]
  fn scale(&mut self, core: &mut Core<Self>, scale_factor: f64) {}
}
