use std::collections::HashMap;

use bytemuck::Zeroable;
use glam::{vec2, vec3, vec4, Quat, Vec2, Vec4};
use tap::Tap;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::client::gameplay::beatmap::Beatmap;

use apex_framework::{
  graphics::{
    bindable::Bindable,
    camera::{Camera as _, Camera2D, ProjectionOrthographic},
    color::Color,
    drawable::Drawable,
    instance::Instance,
    quad_vertex::QuadVertex,
    scene::Scene,
    texture::Texture,
    uniform::Uniform,
  },
  time::time::Time,
};

use super::hit_object_model::{BakedHitObjectModel, HitObjectModel};

#[derive(Debug, Clone)]
pub struct TaikoRendererConfig {
  // Graphics
  pub width: f32,
  pub height: f32,
  pub scale_factor: f32,

  // Taiko
  pub gameplay_scale: f64,
  pub conveyor_zoom: f64,
  pub hit_position_x: f32,
  pub hit_position_y: f32,
  pub don: Color,
  pub kat: Color,

  pub hit_animation_height: f64,
}

#[rustfmt::skip]
pub struct TaikoRenderer {
  pub scene: Scene<ProjectionOrthographic, Camera2D>,

  pub pipeline: wgpu::RenderPipeline,
  pub pipeline_layout: wgpu::PipelineLayout,
  pub shader: wgpu::ShaderModule,

  pub time_uniform: Uniform<Vec4>,

  pub texture_layout           : wgpu::BindGroupLayout,
  pub circle_texture           : Texture,
  pub finisher_texture         : Texture,
  pub circle_overlay_texture   : Texture,
  pub finisher_overlay_texture : Texture,

  pub vertex_buffer      : wgpu::Buffer,
  pub vertex_buffer_data : Vec<QuadVertex>,

  pub instance_buffer : wgpu::Buffer,
  pub instances       : Vec<HitObjectModel>,

  pub config: TaikoRendererConfig,
  pub current_beatmap: Beatmap,
}

impl TaikoRenderer {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    config: TaikoRendererConfig,
  ) -> Self {
    let scene = Self::create_scene(
      device,
      config.width,
      config.height,
      config.scale_factor,
      config.hit_position_x,
      config.hit_position_y,
    );

    let time_uniform = Uniform::new(device, wgpu::ShaderStages::VERTEX);
    let shader = Self::create_shader(device);
    let texture_layout = Self::create_texture_layout(device);
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[
        scene.layout(),
        time_uniform.layout(),
        &texture_layout,
        &texture_layout,
        &texture_layout,
        &texture_layout,
      ],
      push_constant_ranges: &[],
    });

    let pipeline = Self::create_pipeline(device, &pipeline_layout, &shader, format, &config);

    let vertex_buffer_data = QuadVertex::vertices_quad_cww(-0.5, 0.5);
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&vertex_buffer_data),
      usage: wgpu::BufferUsages::VERTEX,
    });

    // Circle instances
    let instances = vec![];
    let instance_data = instances.iter().map(Instance::bake).collect::<Vec<_>>();
    let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Instance Buffer"),
      contents: bytemuck::cast_slice(&instance_data),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    let circle_texture = Texture::from_path("./assets/taikohitcircle.png", device, queue).unwrap();
    let finisher_texture = Texture::from_path("./assets/taikobigcircle.png", device, queue).unwrap();
    let circle_overlay_texture = Texture::from_path("./assets/taikohitcircleoverlay.png", device, queue).unwrap();
    let finisher_overlay_texture = Texture::from_path("./assets/taikobigcircleoverlay.png", device, queue).unwrap();

    let current_beatmap = Beatmap::default();

    let mut renderer = Self {
      scene,

      pipeline,
      pipeline_layout,
      shader,

      time_uniform,

      texture_layout,
      circle_texture,
      finisher_texture,
      circle_overlay_texture,
      finisher_overlay_texture,

      vertex_buffer,
      vertex_buffer_data,

      instance_buffer,
      instances,

      config,
      current_beatmap,
    };

    renderer.update_camera(queue);

    return renderer;
  }

  fn create_scene(
    device: &wgpu::Device,
    width: f32,
    height: f32,
    scale_factor: f32,
    hit_position_x: f32,
    hit_position_y: f32,
  ) -> Scene<ProjectionOrthographic, Camera2D> {
    return Scene::<ProjectionOrthographic, Camera2D> {
      projection: ProjectionOrthographic::new(width * scale_factor, height * scale_factor, -100.0, 100.0),
      camera: Camera2D::new(
        vec3(hit_position_x, hit_position_y, -50.0),
        Quat::zeroed(),
        vec3(scale_factor, scale_factor, 1.0),
      ),
      uniform: Uniform::new(device, wgpu::ShaderStages::VERTEX),
    };
  }

  fn create_texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
      label: Some("default_texture_bind_group_layout"),
    });
  }

  fn create_shader(device: &wgpu::Device) -> wgpu::ShaderModule {
    return device.create_shader_module(wgpu::include_wgsl!("taiko_shader.wgsl"));
  }

  fn create_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    config: &TaikoRendererConfig,
  ) -> wgpu::RenderPipeline {
    return device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(pipeline_layout),

      vertex: wgpu::VertexState {
        module: shader,
        entry_point: "vs_main",
        buffers: &[QuadVertex::describe(), HitObjectModel::describe()],
        compilation_options: wgpu::PipelineCompilationOptions {
          constants: &HashMap::new().tap_mut(|x| {
            x.insert(String::from("1000"), config.hit_animation_height);
          }),
          ..Default::default()
        },
      },
      fragment: Some(wgpu::FragmentState {
        module: shader,
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
  }
}

impl TaikoRenderer {
  pub fn prepare(&mut self, queue: &wgpu::Queue, time: Time) {
    // Update time uniform
    let time_offset = time.to_seconds() * 1000.0 * self.config.conveyor_zoom * -1.0;
    self.time_uniform.update(queue, &vec4(time_offset as f32, 0.0, 0.0, 0.0));
  }

  pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
    rpass.set_pipeline(&self.pipeline);

    self.scene.bind(rpass, 0);
    self.time_uniform.bind(rpass, 1);
    rpass.set_bind_group(2, &self.circle_texture.bind_group, &[]);
    rpass.set_bind_group(3, &self.circle_overlay_texture.bind_group, &[]);
    rpass.set_bind_group(4, &self.finisher_texture.bind_group, &[]);
    rpass.set_bind_group(5, &self.finisher_overlay_texture.bind_group, &[]);

    rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
    rpass.draw(0 .. self.vertex_buffer_data.len() as u32, 0 .. self.instances.len() as u32);

    // honestly, culling does not affect performance that much so don't bother for now
    // rpass.draw(0 .. self.vertex_buffer_data.len() as u32, 0 .. (self.instances.len() - self.culling) as u32);
  }
}

impl TaikoRenderer {
  pub fn set_hit(&mut self, queue: &wgpu::Queue, hit_idx: usize, hit_time: Time) {
    let len = self.instances.len();
    let idx = len - hit_idx - 1;
    let instance = &mut self.instances[idx];
    instance.hit = hit_time * 1000.0 * self.config.conveyor_zoom * -1.0;

    let single_baked = [instance.bake()];
    let byte_slice: &[u8] = bytemuck::cast_slice(&single_baked);
    let offset = (std::mem::size_of::<BakedHitObjectModel>() * idx) as wgpu::BufferAddress;

    queue.write_buffer(&self.instance_buffer, offset, byte_slice);
  }

  pub fn set_hit_all(&mut self, queue: &wgpu::Queue) {
    for instance in self.instances.iter_mut() {
      instance.hit = Time::from_seconds(instance.time * -1.0);
    }

    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    let byte_slice: &[u8] = bytemuck::cast_slice(&instance_data);
    let offset = 0 as wgpu::BufferAddress;

    queue.write_buffer(&self.instance_buffer, offset, byte_slice);
  }

  pub fn restart_beatmap(&mut self, queue: &wgpu::Queue) {
    for inst in &mut self.instances {
      inst.hit = Time::zero();
    }

    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    let byte_slice = bytemuck::cast_slice(&instance_data);
    let offset = 0 as wgpu::BufferAddress;

    queue.write_buffer(&self.instance_buffer, offset, byte_slice);
  }

  pub fn set_hit_scoped(&mut self, queue: &wgpu::Queue, f: impl FnOnce(&mut [HitObjectModel], f32)) {
    let multiplier = 1000.0 * self.config.conveyor_zoom as f32 * -1.0;
    f(&mut self.instances, multiplier);

    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    let byte_slice = bytemuck::cast_slice(&instance_data);
    let offset = 0 as wgpu::BufferAddress;

    queue.write_buffer(&self.instance_buffer, offset, byte_slice);
  }

  pub fn load_beatmap(&mut self, device: &wgpu::Device, beatmap: Beatmap) {
    self.current_beatmap = beatmap;
    self.prepare_instances(device);
  }

  pub fn set_hit_position_x(&mut self, queue: &wgpu::Queue, value: f32) {
    self.config.hit_position_x = value;
    self.update_camera(queue);
  }

  pub fn set_hit_position_y(&mut self, queue: &wgpu::Queue, value: f32) {
    self.config.hit_position_y = value;
    self.update_camera(queue);
  }

  pub fn set_conveyor_zoom(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f64) {
    self.config.conveyor_zoom = value;
    self.update_camera(queue);
    self.prepare_instances(device);
  }

  pub fn set_gameplay_scale(&mut self, queue: &wgpu::Queue, value: f64) {
    self.config.gameplay_scale = value;
    self.update_camera(queue);
  }

  pub fn set_don_color(&mut self, device: &wgpu::Device, value: Color) {
    self.config.don = value;
    self.prepare_instances(device);
  }

  pub fn set_kat_color(&mut self, device: &wgpu::Device, value: Color) {
    self.config.kat = value;
    self.prepare_instances(device);
  }

  pub fn set_hit_animation_height(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat, value: f64) {
    self.config.hit_animation_height = value;
    self.recreate_pipeline(device, format);
  }
}

impl TaikoRenderer {
  fn recreate_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
    self.pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&self.pipeline_layout),

      vertex: wgpu::VertexState {
        module: &self.shader,
        entry_point: "vs_main",
        buffers: &[QuadVertex::describe(), HitObjectModel::describe()],
        compilation_options: wgpu::PipelineCompilationOptions {
          constants: &HashMap::new().tap_mut(|x| {
            x.insert(String::from("1000"), self.config.hit_animation_height);
          }),
          ..Default::default()
        },
      },
      fragment: Some(wgpu::FragmentState {
        module: &self.shader,
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
  }

  fn prepare_instances(&mut self, device: &wgpu::Device) {
    const OSU_TAIKO_VELOCITY_MULTIPLIER: f64 = 1.4;
    const OSU_TAIKO_CIRCLE_SIZE: f32 = 128.0;

    let circle_size = OSU_TAIKO_CIRCLE_SIZE;

    self.instances.clear();

    let mut idx_t = self.current_beatmap.timing_points.len() - 1;
    let mut idx_v = self.current_beatmap.velocity_points.len() - 1;
    for obj in self.current_beatmap.hit_objects.iter().rev() {
      #[rustfmt::skip] {
        while self.current_beatmap.timing_points[idx_t].time > obj.time && idx_t != 0 { idx_t -= 1; }
        while self.current_beatmap.velocity_points[idx_v].time > obj.time && idx_v != 0 { idx_v -= 1; }
      };

      // Timing
      let beat_length = 60.0 / self.current_beatmap.timing_points[idx_t].bpm * 1000.0; // we want ms...
      let velocity = self.current_beatmap.velocity_points[idx_v].velocity;

      let base_length = 1000.0;

      #[rustfmt::skip]
      let multiplier = OSU_TAIKO_VELOCITY_MULTIPLIER * velocity * base_length / beat_length * self.current_beatmap.velocity_multiplier as f64;

      let size_big = vec2(circle_size * 1.55, circle_size * 1.55);
      let size_small = vec2(circle_size, circle_size);

      self.instances.push(HitObjectModel {
        time: (obj.time.to_seconds() * 1000.0 * self.config.conveyor_zoom) as f32,
        size: if obj.big { size_big } else { size_small },
        color: if obj.color.is_kat() { self.config.kat } else { self.config.don },
        finisher: obj.big,
        velocity: multiplier as f32,
        hit: Time::zero(),
      });
    }

    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Instance Buffer"),
      contents: bytemuck::cast_slice(&instance_data),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });
  }

  fn update_camera(&mut self, queue: &wgpu::Queue) {
    // Update scene matrix
    let camera_scale = self.config.scale_factor * self.config.gameplay_scale as f32;
    self.scene.camera.set_scale(Vec2::splat(camera_scale).extend(1.0));
    self.scene.camera.set_x(self.config.hit_position_x / self.config.gameplay_scale as f32);
    self.scene.camera.set_y(self.config.hit_position_y / self.config.gameplay_scale as f32);
    self.scene.update(queue);
  }
}

impl Drawable for TaikoRenderer {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    self.scene = Self::create_scene(
      device,
      self.config.width,
      self.config.height,
      self.config.scale_factor,
      self.config.hit_position_x,
      self.config.hit_position_y,
    );

    self.time_uniform = Uniform::new(device, wgpu::ShaderStages::VERTEX);
    self.shader = Self::create_shader(device);
    self.texture_layout = Self::create_texture_layout(device);
    self.pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[
        self.scene.layout(),
        self.time_uniform.layout(),
        &self.texture_layout,
        &self.texture_layout,
        &self.texture_layout,
        &self.texture_layout,
      ],
      push_constant_ranges: &[],
    });

    self.pipeline = Self::create_pipeline(device, &self.pipeline_layout, &self.shader, format, &self.config);

    let vertex_buffer_data = QuadVertex::vertices_quad_cww(-0.5, 0.5);
    self.vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Vertex Buffer"),
      contents: bytemuck::cast_slice(&vertex_buffer_data),
      usage: wgpu::BufferUsages::VERTEX,
    });

    // Circle instances
    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    self.instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("Instance Buffer"),
      contents: bytemuck::cast_slice(&instance_data),
      usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
    });

    self.circle_texture = Texture::from_path("./assets/taikohitcircle.png", device, queue).unwrap();
    self.finisher_texture = Texture::from_path("./assets/taikobigcircle.png", device, queue).unwrap();
    self.circle_overlay_texture = Texture::from_path("./assets/taikohitcircleoverlay.png", device, queue).unwrap();
    self.finisher_overlay_texture = Texture::from_path("./assets/taikobigcircleoverlay.png", device, queue).unwrap();

    self.update_camera(queue);
  }

  fn resize(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, width: f32, height: f32) {
    self.config.width = width;
    self.config.height = height;

    self.scene.resize(PhysicalSize::new(
      (self.config.width * self.config.scale_factor) as u32,
      (self.config.height * self.config.scale_factor) as u32,
    ));

    self.update_camera(queue);
  }

  fn resize_width(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, value: f32) {
    self.config.width = value;

    self.scene.resize(PhysicalSize::new(
      (self.config.width * self.config.scale_factor) as u32,
      (self.config.height * self.config.scale_factor) as u32,
    ));

    self.update_camera(queue);
  }

  fn resize_height(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, value: f32) {
    self.config.height = value;

    self.scene.resize(PhysicalSize::new(
      (self.config.width * self.config.scale_factor) as u32,
      (self.config.height * self.config.scale_factor) as u32,
    ));

    self.update_camera(queue);
  }

  fn rescale(&mut self, _device: &wgpu::Device, queue: &wgpu::Queue, value: f32) {
    self.config.scale_factor = value;
    self.update_camera(queue);
  }
}
