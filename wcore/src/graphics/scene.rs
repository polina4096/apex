use cgmath::Matrix4;

use crate::graphics::{camera::{Projection, Camera, Transformation}, uniform::Uniform, bindable::Bindable};

use super::layout::Layout;

pub struct Scene<P: Projection, C: Camera> {
    pub projection : P,
    pub camera     : C,
    pub uniform    : Uniform<Matrix4<f32>>
}

impl<P: Projection, C: Camera> Scene<P, C> {
    pub fn update(&self, queue: &wgpu::Queue) {
        self.uniform.update(queue, &self.apply());
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
    fn apply(&self) -> Matrix4<f32> {
        return self.projection.apply() * self.camera.apply();
    }
}