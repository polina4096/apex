use glam::{vec2, Vec2};
use instant::Instant;
use wgpu::{util::DeviceExt as _, PipelineCompilationOptions};
use winit::dpi::PhysicalSize;

use crate::graphics::{bindable::Bindable, drawable::Drawable, quad_vertex::QuadVertex, uniform::Uniform};

pub struct Framebuffer {
  // Resources used to render the framebuffer
  pipeline: wgpu::RenderPipeline,
  vertex_buffer: wgpu::Buffer,
  vertex_buffer_data: Vec<QuadVertex>,

  /// The framebuffer texture, which will be rendered to by the renderers
  texture: wgpu::Texture,
  texture_format: wgpu::TextureFormat,
  texture_view: wgpu::TextureView,
  texture_bind_group: wgpu::BindGroup,
  texture_bind_group_layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,

  fade_time_uniform: Uniform<f32>,
  border_thickness_uniform: Uniform<Vec2>,
  last_scale_change: Instant,

  size: PhysicalSize<u32>,
  scale_factor: f32,
  scale: Vec2,
}

impl Framebuffer {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    size: PhysicalSize<u32>,
    scale_factor: f32,
  ) -> Self {
    let texture = Self::create_texture(device, size.width, size.height, format);
    let texture_view = Self::create_texture_view(&texture);
    let sampler = Self::create_sampler(device);

    let texture_bind_group_layout = Self::create_bind_group_layout(device);
    let texture_bind_group = Self::create_bind_group(device, &texture_bind_group_layout, &texture_view, &sampler);

    let fade_time_uniform = Uniform::new(device, wgpu::ShaderStages::FRAGMENT);
    let border_thickness_uniform = Uniform::new(device, wgpu::ShaderStages::FRAGMENT);

    fade_time_uniform.update(queue, &2.0);
    border_thickness_uniform.update(queue, &vec2(2.0 / size.width as f32, 2.0 / size.height as f32));

    let shader = Self::create_shader(device);
    let pipeline_layout = Self::create_pipeline_layout(
      device,
      &[
        &texture_bind_group_layout,
        fade_time_uniform.layout(),
        border_thickness_uniform.layout(),
      ],
    );
    let pipeline = Self::create_render_pipeline(device, &pipeline_layout, &shader, format);

    let vertex_buffer_data = Self::create_vertex_buffer_data(Vec2::splat(1.0));
    let vertex_buffer = Self::create_vertex_buffer(device, &vertex_buffer_data);

    return Self {
      pipeline,
      vertex_buffer,
      vertex_buffer_data,
      texture,
      texture_format: format,
      texture_view,
      texture_bind_group,
      texture_bind_group_layout,
      sampler,
      fade_time_uniform,
      border_thickness_uniform,
      last_scale_change: Instant::now(),
      size,
      scale_factor,
      scale: Vec2::splat(1.0),
    };
  }

  fn create_texture(device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat) -> wgpu::Texture {
    return device.create_texture(&wgpu::TextureDescriptor {
      label: Some("framebuffer"),
      size: wgpu::Extent3d {
        width: width,
        height: height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: format,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    });
  }

  fn create_texture_view(texture: &wgpu::Texture) -> wgpu::TextureView {
    return texture.create_view(&wgpu::TextureViewDescriptor {
      label: Some("framebuffer_texture_view"),
      ..Default::default()
    });
  }

  fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
    return device.create_sampler(&wgpu::SamplerDescriptor {
      address_mode_u: wgpu::AddressMode::ClampToEdge,
      address_mode_v: wgpu::AddressMode::ClampToEdge,
      address_mode_w: wgpu::AddressMode::ClampToEdge,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::FilterMode::Linear,
      ..Default::default()
    });
  }

  fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    return device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      entries: &[
        wgpu::BindGroupLayoutEntry {
          binding: 0,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Texture {
            multisampled: false,
            view_dimension: wgpu::TextureViewDimension::D2,
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
          },
          count: None,
        },
        wgpu::BindGroupLayoutEntry {
          binding: 1,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
      label: Some("framebuffer_bind_group_layout"),
    });
  }

  fn create_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
  ) -> wgpu::BindGroup {
    return device.create_bind_group(&wgpu::BindGroupDescriptor {
      layout: bind_group_layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureView(texture_view),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(sampler),
        },
      ],
      label: Some("framebuffer_bind_group"),
    });
  }

  fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layouts: &[&wgpu::BindGroupLayout],
  ) -> wgpu::PipelineLayout {
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("framebuffer_render_pipeline_layout"),
      bind_group_layouts,
      push_constant_ranges: &[],
    })
  }

  fn create_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    return device.create_shader_module(wgpu::include_wgsl!("framebuffer.wgsl"));
  }

  fn create_render_pipeline(
    device: &wgpu::Device,
    render_pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
  ) -> wgpu::RenderPipeline {
    return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("framebuffer_render_pipeline"),
      layout: Some(render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: shader,
        entry_point: "vs_main",
        buffers: &[QuadVertex::describe()],
        compilation_options: PipelineCompilationOptions::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: format,
          blend: Some(wgpu::BlendState::ALPHA_BLENDING),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: PipelineCompilationOptions::default(),
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
        polygon_mode: wgpu::PolygonMode::Fill,
        // Requires Features::DEPTH_CLIP_CONTROL
        unclipped_depth: false,
        // Requires Features::CONSERVATIVE_RASTERIZATION
        conservative: false,
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
      cache: None,
    });
  }

  fn create_vertex_buffer_data(scale: Vec2) -> Vec<QuadVertex> {
    return QuadVertex::vertices_quad_xy_cw(scale.x, scale.y);
  }

  fn create_vertex_buffer(device: &wgpu::Device, data: &[QuadVertex]) -> wgpu::Buffer {
    return device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("framebuffer_vertex_buffer"),
      contents: bytemuck::cast_slice(data),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
  }
}

impl Framebuffer {
  pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
    rpass.set_pipeline(&self.pipeline);
    rpass.set_bind_group(0, &self.texture_bind_group, &[]);
    self.fade_time_uniform.bind(rpass, 1);
    self.border_thickness_uniform.bind(rpass, 2);
    rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    rpass.draw(0 .. self.vertex_buffer_data.len() as u32, 0 .. 1);
  }

  pub fn prepare(&mut self, queue: &wgpu::Queue) {
    self.update_fade_time(queue);
  }

  pub fn render_frame<'rpass>(
    &'rpass self,
    encoder: &'rpass mut wgpu::CommandEncoder,
    descriptor: &wgpu::RenderPassDescriptor,
    f: impl FnOnce(&mut wgpu::RenderPass<'rpass>),
  ) {
    f(&mut encoder.begin_render_pass(descriptor));
  }

  /// Size of the framebuffer from `0.0` to `1.0`.
  pub fn set_scale(&mut self, queue: &wgpu::Queue, scale: Vec2) {
    self.scale = scale;
    self.vertex_buffer_data = Self::create_vertex_buffer_data(self.scale);
    self.rebuild_vertex_buffer(queue);
    self.last_scale_change = Instant::now();
  }

  /// Size of the framebuffer from `0.0` to `1.0`.
  pub fn set_scale_x(&mut self, queue: &wgpu::Queue, scale_x: f32) {
    self.scale.x = scale_x;
    self.vertex_buffer_data = Self::create_vertex_buffer_data(self.scale);
    self.rebuild_vertex_buffer(queue);
    self.last_scale_change = Instant::now();
  }

  /// Size of the framebuffer from `0.0` to `1.0`.
  pub fn set_scale_y(&mut self, queue: &wgpu::Queue, scale_y: f32) {
    self.scale.y = scale_y;
    self.vertex_buffer_data = Self::create_vertex_buffer_data(self.scale);
    self.rebuild_vertex_buffer(queue);
    self.last_scale_change = Instant::now();
  }

  pub fn texture_view(&self) -> &wgpu::TextureView {
    return &self.texture_view;
  }

  fn rebuild_vertex_buffer(&self, queue: &wgpu::Queue) {
    let data = bytemuck::cast_slice(&self.vertex_buffer_data);
    queue.write_buffer(&self.vertex_buffer, 0, data);
  }

  fn update_fade_time(&mut self, queue: &wgpu::Queue) {
    let now = Instant::now();
    let duration = now.duration_since(self.last_scale_change);

    const MAX_FADE_TIME: f32 = 1.0;
    let duration = duration.as_secs_f32();
    if duration < MAX_FADE_TIME * 2.0 {
      self.fade_time_uniform.update(queue, &duration);
    }
  }
}

impl Drawable for Framebuffer {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    self.texture = Self::create_texture(device, self.size.width, self.size.height, format);
    self.texture_view = Self::create_texture_view(&self.texture);
    self.sampler = Self::create_sampler(device);

    self.texture_bind_group_layout = Self::create_bind_group_layout(device);
    self.texture_bind_group = Self::create_bind_group(
      device,
      &self.texture_bind_group_layout,
      &self.texture_view,
      &self.sampler,
      //
    );

    self.fade_time_uniform = Uniform::new(device, wgpu::ShaderStages::FRAGMENT);
    self.border_thickness_uniform = Uniform::new(device, wgpu::ShaderStages::FRAGMENT);

    self.fade_time_uniform.update(queue, &2.0);
    self.border_thickness_uniform.update(
      queue,
      &vec2(
        2.0 / (self.size.width as f32 * self.scale_factor),
        2.0 / (self.size.height as f32 * self.scale_factor),
      ),
    );

    let shader = Self::create_shader(device);
    let pipeline_layout = Self::create_pipeline_layout(
      device,
      &[
        &self.texture_bind_group_layout,
        self.fade_time_uniform.layout(),
        self.border_thickness_uniform.layout(),
      ],
    );

    self.pipeline = Self::create_render_pipeline(device, &pipeline_layout, &shader, format);

    self.vertex_buffer_data = Self::create_vertex_buffer_data(self.scale);
    self.vertex_buffer = Self::create_vertex_buffer(device, &self.vertex_buffer_data);
  }

  fn resize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: f32, height: f32) {
    let p_width = (width * self.scale_factor) as u32;
    let p_height = (height * self.scale_factor) as u32;
    self.size = PhysicalSize::new(p_width, p_height);

    self.border_thickness_uniform.update(queue, &vec2(2.0 / width, 2.0 / height));
    self.texture = Self::create_texture(device, self.size.width, self.size.height, self.texture_format);
    self.texture_bind_group =
      Self::create_bind_group(device, &self.texture_bind_group_layout, &self.texture_view, &self.sampler);
  }

  fn resize_width(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue, value: f32) {
    let p_width = (value * self.scale_factor) as u32;
    self.size.width = p_width;

    self.texture = Self::create_texture(device, self.size.width, self.size.height, self.texture_format);
    self.texture_bind_group =
      Self::create_bind_group(device, &self.texture_bind_group_layout, &self.texture_view, &self.sampler);
  }

  fn resize_height(&mut self, device: &wgpu::Device, _queue: &wgpu::Queue, value: f32) {
    let p_height = (value * self.scale_factor) as u32;
    self.size.height = p_height;

    self.texture = Self::create_texture(device, self.size.width, self.size.height, self.texture_format);
    self.texture_bind_group =
      Self::create_bind_group(device, &self.texture_bind_group_layout, &self.texture_view, &self.sampler);
  }

  fn rescale(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue, value: f32) {
    self.scale_factor = value;
  }
}
