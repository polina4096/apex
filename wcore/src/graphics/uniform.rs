use std::marker::PhantomData;

use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::graphics::bindable::Bindable;

use super::layout::Layout;

pub struct Uniform<T: Clone + Pod + Zeroable> {
    buffer            : wgpu::Buffer,
    bind_group        : wgpu::BindGroup,
    bind_group_layout : wgpu::BindGroupLayout,
    _0                : PhantomData<T>,
}

impl<T: Clone + Pod + Zeroable> Uniform<T> {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::bytes_of(&T::zeroed()),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
        });

        return Self {
            buffer,
            bind_group,
            bind_group_layout,
            _0: Default::default()
        };
    }

    /// Loads uniform with a new value.
    /// The value is updated at the end of the RenderPass.
    pub fn update(&self, queue: &wgpu::Queue, value: &T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(value));
    }
}

impl<T: Clone + Pod + Zeroable> Bindable for Uniform<T> {
    fn bind<'pass, 'uniform: 'pass>(&'uniform self, render_pass: &mut wgpu::RenderPass<'pass>, index: u32) {
        render_pass.set_bind_group(index, &self.bind_group, &[]);
    }

    fn group(&self) -> &wgpu::BindGroup {
        return &self.bind_group;
    }
}

impl<T: Clone + Pod + Zeroable> Layout for Uniform<T> {
    fn layout(&self) -> &wgpu::BindGroupLayout {
        return &self.bind_group_layout;
    }
}