use glam::{Mat4, Vec3};

use super::{bindable::Bindable, camera::{Camera, Projection, Transformation}, layout::Layout, uniform::Uniform};

pub struct Scene<P: Projection, C: Camera> {
  pub projection : P,
  pub camera     : C,
  pub uniform    : Uniform<Mat4>
}

impl<P: Projection, C: Camera> Scene<P, C> {
  pub fn update(&self, queue: &wgpu::Queue) {
    self.uniform.update(queue, &self.apply());
  }

  pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
    self.projection.resize(size.width, size.height);
  }

  pub fn scale(&mut self, scale_factor: f64) {
    self.camera.set_scale(Vec3::new(scale_factor as f32, scale_factor as f32, scale_factor as f32));
  }
}

impl<P: Projection, C: Camera> Bindable for Scene<P, C> {
  fn bind<'pass, 'uniform: 'pass>(&'uniform self, render_pass: &mut wgpu::RenderPass<'pass>, index: u32) {
    self.uniform.bind(render_pass, index);
  }

  fn group(&self) -> &wgpu::BindGroup {
    return self.uniform.group();
  }
}

impl<P: Projection, C: Camera> Layout for Scene<P, C> {
  fn layout(&self) -> &wgpu::BindGroupLayout {
    return self.uniform.layout();
  }
}

impl<P: Projection, C: Camera> Transformation for Scene<P, C> {
  fn apply(&self) -> Mat4 {
    return self.projection.apply() * self.camera.apply();
  }
}
