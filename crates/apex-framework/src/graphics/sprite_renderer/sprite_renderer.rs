use bytemuck::Zeroable;
use glam::{vec2, vec3, Quat, Vec2};
use guillotiere::{AtlasAllocator, Size};
use image::{DynamicImage, GenericImage, GenericImageView as _};
use tap::Tap;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::graphics::{
  bindable::Bindable,
  camera::{Camera as _, Camera2D, ProjectionOrthographic},
  color::Color,
  drawable::Drawable,
  instance::Instance,
  origin::Origin,
  quad_vertex::QuadVertex,
  scene::Scene,
  texture::Texture,
  uniform::Uniform,
};

use super::sprite_model::SpriteModel;

pub type AllocId = guillotiere::AllocId;

pub struct SpriteRenderer {
  scene: Scene<ProjectionOrthographic, Camera2D>,

  pipeline: wgpu::RenderPipeline,

  atlas_allocator: AtlasAllocator,
  atlas_texture: Texture,
  atlas_image: DynamicImage,

  vertex_buffer: wgpu::Buffer,
  vertex_buffer_data: Vec<QuadVertex>,

  instance_buffer: wgpu::Buffer,
  instances: Vec<SpriteModel>,

  width: f32,
  height: f32,
  scale_factor: f32,
}

impl SpriteRenderer {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    width: f32,
    height: f32,
    scale_factor: f32,
  ) -> Self {
    #[rustfmt::skip]
    let scene = Self::create_scene(device, width, height, scale_factor);

    let dim = device.limits().max_texture_dimension_2d;
    let atlas_allocator = AtlasAllocator::new(guillotiere::size2(dim as i32, dim as i32));
    let atlas_texture = Texture::dummy(device, queue);
    let atlas_image = DynamicImage::new_rgba8(dim, dim);

    let shader = device.create_shader_module(wgpu::include_wgsl!("sprite_shader.wgsl"));
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[scene.layout(), &atlas_texture.bind_group_layout],
      push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&pipeline_layout),

      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[QuadVertex::describe(), SpriteModel::describe()],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: format,
          blend: Some(wgpu::BlendState::ALPHA_BLENDING),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      }),

      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
        strip_index_format: None,
      },

      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },

      depth_stencil: None,
      multiview: None,
      cache: None,
    });

    let vertex_buffer_data = QuadVertex::vertices_quad_cww(-1.0, 1.0);
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&vertex_buffer_data),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let instances = vec![];
    let instance_data = instances.iter().map(Instance::bake).collect::<Vec<_>>();
    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Instance Buffer"),
      contents: bytemuck::cast_slice(&instance_data),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    let renderer = Self {
      scene,
      pipeline,
      atlas_allocator,
      atlas_texture,
      atlas_image,
      vertex_buffer,
      vertex_buffer_data,
      instance_buffer,
      instances,
      width,
      height,
      scale_factor,
    };

    return renderer.tap_mut(|x| x.update_camera(queue));
  }

  fn create_scene(
    device: &wgpu::Device,
    width: f32,
    height: f32,
    scale_factor: f32,
  ) -> Scene<ProjectionOrthographic, Camera2D> {
    return Scene::<ProjectionOrthographic, Camera2D> {
      projection: ProjectionOrthographic::new(width * scale_factor, height * scale_factor, -100.0, 100.0),
      camera: Camera2D::new(vec3(0.0, 0.0, -50.0), Quat::zeroed(), vec3(scale_factor, scale_factor, 1.0)),
      uniform: Uniform::new(device, wgpu::ShaderStages::VERTEX),
    };
  }
}

impl SpriteRenderer {
  /// Don't forget to call `update_atlas_texture` afterwards, or you'll get a stale texture.
  pub fn add_texture(&mut self, image: &DynamicImage) -> AllocId {
    let (width, height) = image.dimensions();
    let rect = self.atlas_allocator.allocate(Size::new(width as i32, height as i32)).unwrap();

    let x = rect.rectangle.min.x as u32;
    let y = rect.rectangle.min.y as u32;
    self.atlas_image.copy_from(image, x, y).unwrap();

    return rect.id;
  }

  /// Don't forget to call `update_atlas_texture` afterwards, or you'll get a stale texture.
  pub fn add_textures<const N: usize>(&mut self, images: [&DynamicImage; N]) -> [AllocId; N] {
    let mut ids = [AllocId::deserialize(0); N];

    for (idx, image) in images.iter().copied().enumerate() {
      let (width, height) = image.dimensions();
      let rect = self.atlas_allocator.allocate(Size::new(width as i32, height as i32)).unwrap();

      let x = rect.rectangle.min.x as u32;
      let y = rect.rectangle.min.y as u32;
      self.atlas_image.copy_from(image, x, y).unwrap();
      ids[idx] = rect.id;
    }

    return ids;
  }

  /// Uploads the current atlas image to the GPU.
  pub fn update_atlas_texture(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    self.atlas_texture = Texture::from_image(&self.atlas_image, device, queue);
  }

  pub fn uv_pairs(&self, texture: AllocId) -> (Vec2, Vec2) {
    let rect = self.atlas_allocator[texture];
    let (atlas_width, atlas_height) = self.atlas_image.dimensions();
    let x = rect.min.x as f32 / atlas_width as f32;
    let y = rect.min.y as f32 / atlas_height as f32;
    let w = rect.width() as f32 / atlas_width as f32;
    let h = rect.height() as f32 / atlas_height as f32;

    return (vec2(x, y), vec2(w, h));
  }

  pub fn alloc_sprite(
    &mut self,
    device: &wgpu::Device,
    pos: Vec2,
    size: Vec2,
    origin: Origin,
    flip_x: bool,
    flip_y: bool,
    texture: AllocId,
  ) -> usize {
    let rect = self.atlas_allocator[texture];
    let (atlas_width, atlas_height) = self.atlas_image.dimensions();
    let mut x = rect.min.x as f32 / atlas_width as f32;
    let mut y = rect.min.y as f32 / atlas_height as f32;
    let mut w = rect.width() as f32 / atlas_width as f32;
    let mut h = rect.height() as f32 / atlas_height as f32;

    if flip_x {
      x += w;
      w = -w;
    }

    if flip_y {
      y += h;
      h = -h;
    }

    let idx = self.instances.len();
    self.instances.push(SpriteModel {
      position: pos,
      origin,
      scale: vec2(size.x, size.y),
      rotation: Quat::zeroed(),
      color: Color::from_rgb(255, 255, 255),
      uv_offset: vec2(x, y),
      uv_scale: vec2(w, h),
    });

    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Instance Buffer"),
      contents: bytemuck::cast_slice(&instance_data),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    return idx;
  }

  pub fn mutate_sprite(&mut self, device: &wgpu::Device, idx: usize, f: impl FnOnce(&mut SpriteModel)) {
    let Some(instance) = self.instances.get_mut(idx) else {
      return;
    };

    f(instance);

    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Instance Buffer"),
      contents: bytemuck::cast_slice(&instance_data),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
  }
}

impl SpriteRenderer {
  pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
    rpass.set_pipeline(&self.pipeline);

    self.scene.bind(rpass, 0);
    self.atlas_texture.bind(rpass, 1);

    rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    rpass.draw(0 .. self.vertex_buffer_data.len() as u32, 0 .. self.instances.len() as u32);
  }

  fn update_camera(&mut self, queue: &wgpu::Queue) {
    // Update scene matrix
    self.scene.camera.set_scale(vec3(self.scale_factor, self.scale_factor, 1.0));
    self.scene.update(queue);
  }
}

impl Drawable for SpriteRenderer {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    self.scene = Self::create_scene(device, self.width, self.height, self.scale_factor);
    let shader = device.create_shader_module(wgpu::include_wgsl!("sprite_shader.wgsl"));
    self.atlas_texture = Texture::from_image(&self.atlas_image, device, queue);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[self.scene.layout(), &self.atlas_texture.bind_group_layout],
      push_constant_ranges: &[],
    });

    self.pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&pipeline_layout),

      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[QuadVertex::describe(), SpriteModel::describe()],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: "fs_main",
        targets: &[Some(wgpu::ColorTargetState {
          format: format,
          blend: Some(wgpu::BlendState::ALPHA_BLENDING),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: wgpu::PipelineCompilationOptions::default(),
      }),

      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
        strip_index_format: None,
      },

      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },

      depth_stencil: None,
      multiview: None,
      cache: None,
    });

    self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&self.vertex_buffer_data),
      usage: wgpu::BufferUsages::VERTEX,
    });

    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Instance Buffer"),
      contents: bytemuck::cast_slice(&instance_data),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    self.update_camera(queue);
  }

  fn resize(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, width: f32, height: f32) {
    self.width = width;
    self.height = height;

    let p_width = (self.width * self.scale_factor) as u32;
    let p_height = (self.height * self.scale_factor) as u32;
    self.scene.resize(PhysicalSize::new(p_width, p_height));

    self.update_camera(queue);
  }

  fn resize_width(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, value: f32) {
    self.width = value;

    let p_width = (self.width * self.scale_factor) as u32;
    let p_height = (self.height * self.scale_factor) as u32;
    self.scene.resize(PhysicalSize::new(p_width, p_height));

    self.update_camera(queue);
  }

  fn resize_height(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, value: f32) {
    self.height = value;

    let p_width = (self.width * self.scale_factor) as u32;
    let p_height = (self.height * self.scale_factor) as u32;
    self.scene.resize(PhysicalSize::new(p_width, p_height));

    self.update_camera(queue);
  }

  fn rescale(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, value: f32) {
    self.scale_factor = value;
    self.update_camera(queue);
  }
}
