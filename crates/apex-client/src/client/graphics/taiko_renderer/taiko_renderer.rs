use std::{collections::HashMap, path::PathBuf};

use bytemuck::Zeroable;
use glam::{vec2, vec3, vec4, Quat, Vec4};
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
    layout::Layout,
    quad_renderer::data::quad_vertex::QuadVertex,
    scene::Scene,
    texture::Texture,
    uniform::Uniform,
  },
  time::time::Time,
};

use super::data::hit_object_model::{BakedHitObjectModel, HitObjectModel};

pub struct TaikoRendererConfig {
  // Graphics
  pub width: u32,
  pub height: u32,
  pub scale_factor: f64,

  // Taiko
  pub scale: f64,
  pub zoom: f64,
  pub hit_position_x: f32,
  pub hit_position_y: f32,
  pub don: Color,
  pub kat: Color,

  pub hit_height: f64,
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
  pub beatmap: Beatmap, // TODO: make this some other type which has less useless info
}

impl TaikoRenderer {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    config: TaikoRendererConfig,
  ) -> Self {
    #[rustfmt::skip]
    let scene = Scene::<ProjectionOrthographic, Camera2D> {
      projection : ProjectionOrthographic::new(config.width, config.height, -100.0, 100.0),
      camera     : Camera2D::new(vec3(0.0, 0.0, -50.0), Quat::zeroed(), vec3(config.scale_factor as f32, config.scale_factor as f32, 1.0)),
      uniform    : Uniform::new(device),
    };

    let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    let time_uniform = Uniform::new(device);

    let shader = device.create_shader_module(wgpu::include_wgsl!("taiko_shader.wgsl"));
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

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&pipeline_layout),

      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: "vs_main",
        buffers: &[QuadVertex::describe(), HitObjectModel::describe()],
        compilation_options: wgpu::PipelineCompilationOptions {
          constants: &HashMap::new().tap_mut(|x| {
            x.insert(String::from("1000"), config.hit_height);
          }),
          ..Default::default()
        },
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
        polygon_mode: wgpu::PolygonMode::Fill, // Others require Features::NON_FILL_POLYGON_MODE
        unclipped_depth: false,                // Requires Features::DEPTH_CLIP_CONTROL
        conservative: false,                   // Requires Features::CONSERVATIVE_RASTERIZATION
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

    let vertex_buffer_data = QuadVertex::vertices_quad(-0.5, 0.5);
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
    let finisher_overlaytexture = Texture::from_path("./assets/taikobigcircleoverlay.png", device, queue).unwrap();

    let beatmap = Beatmap {
      hit_objects: vec![],
      timing_points: vec![],
      velocity_points: vec![],
      break_points: vec![],
      overall_difficulty: 0.0,
      velocity_multiplier: 0.0,
      audio: PathBuf::new(),
    };

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
      finisher_overlay_texture: finisher_overlaytexture,

      vertex_buffer,
      vertex_buffer_data,

      instance_buffer,
      instances,

      config,
      beatmap,
    };

    renderer.update_camera(queue);

    return renderer;
  }

  pub fn prepare(&mut self, queue: &wgpu::Queue, time: Time) {
    // Update time uniform
    let time_offset = time.to_seconds() * 1000.0 * self.config.zoom * -1.0;
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

  pub fn set_hit(&mut self, queue: &wgpu::Queue, hit_idx: usize, hit_time: Time) {
    let len = self.instances.len();
    let idx = len - hit_idx - 1;
    let instance = &mut self.instances[idx];
    instance.hit = hit_time * 1000.0 * self.config.zoom * -1.0;

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
    let multiplier = 1000.0 * self.config.zoom as f32 * -1.0;
    f(&mut self.instances, multiplier);

    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    let byte_slice = bytemuck::cast_slice(&instance_data);
    let offset = 0 as wgpu::BufferAddress;

    queue.write_buffer(&self.instance_buffer, offset, byte_slice);
  }

  pub fn load_beatmap(&mut self, device: &wgpu::Device, beatmap: Beatmap) {
    self.beatmap = beatmap;
    self.prepare_instances(device);
  }

  pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
    self.config.width = width;
    self.config.height = height;

    self.scene.resize(PhysicalSize::new(width, height));
    self.update_camera(queue);
  }

  pub fn scale(&mut self, queue: &wgpu::Queue, value: f64) {
    self.config.scale_factor = value;
    self.update_camera(queue);
  }

  pub fn set_hit_position_x(&mut self, queue: &wgpu::Queue, value: f32) {
    self.config.hit_position_x = value;
    self.update_camera(queue);
  }

  pub fn set_hit_position_y(&mut self, queue: &wgpu::Queue, value: f32) {
    self.config.hit_position_y = value;
    self.update_camera(queue);
  }

  pub fn set_zoom(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f64) {
    self.config.zoom = value;
    self.update_camera(queue);
    self.prepare_instances(device);
  }

  pub fn set_scale(&mut self, queue: &wgpu::Queue, value: f64) {
    self.config.scale = value;
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

  pub fn set_hit_height(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat, value: f64) {
    dbg!(value);
    self.config.hit_height = value;
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
            x.insert(String::from("1000"), self.config.hit_height);
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
        polygon_mode: wgpu::PolygonMode::Fill, // Others require Features::NON_FILL_POLYGON_MODE
        unclipped_depth: false,                // Requires Features::DEPTH_CLIP_CONTROL
        conservative: false,                   // Requires Features::CONSERVATIVE_RASTERIZATION
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

    let mut idx_t = self.beatmap.timing_points.len() - 1;
    let mut idx_v = self.beatmap.velocity_points.len() - 1;
    for obj in self.beatmap.hit_objects.iter().rev() {
      #[rustfmt::skip] {
        while self.beatmap.timing_points[idx_t].time > obj.time && idx_t != 0 { idx_t -= 1; }
        while self.beatmap.velocity_points[idx_v].time > obj.time && idx_v != 0 { idx_v -= 1; }
      };

      // Timing
      let beat_length = 60.0 / self.beatmap.timing_points[idx_t].bpm * 1000.0; // we want ms...
      let velocity = self.beatmap.velocity_points[idx_v].velocity;

      let base_length = 1000.0;

      #[rustfmt::skip]
      let multiplier = OSU_TAIKO_VELOCITY_MULTIPLIER * velocity * base_length / beat_length * self.beatmap.velocity_multiplier as f64;

      let size_big = vec2(circle_size * 1.55, circle_size * 1.55);
      let size_small = vec2(circle_size, circle_size);

      self.instances.push(HitObjectModel {
        time: (obj.time.to_seconds() * 1000.0 * self.config.zoom) as f32,
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
    let camera_scale = (self.config.scale_factor * self.config.scale) as f32;
    self.scene.camera.set_scale(vec3(camera_scale, camera_scale, 1.0));
    self.scene.camera.set_x(self.config.hit_position_x / self.config.scale as f32);
    self.scene.camera.set_y(self.config.hit_position_y / self.config.scale as f32);
    self.scene.update(queue);
  }
}

impl Drawable for TaikoRenderer {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) {
    #[rustfmt::skip] {
      self.scene = Scene::<ProjectionOrthographic, Camera2D> {
        projection: ProjectionOrthographic::new(self.config.width, self.config.height, -100.0, 100.0),
        camera: Camera2D::new(vec3(0.0, 0.0, -50.0), Quat::zeroed(), vec3(self.config.scale as f32, self.config.scale as f32, 1.0)),
        uniform: Uniform::new(device),
      };
    };

    self.texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    self.time_uniform = Uniform::new(device);

    self.shader = device.create_shader_module(wgpu::include_wgsl!("taiko_shader.wgsl"));
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

    self.pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&self.pipeline_layout),

      vertex: wgpu::VertexState {
        module: &self.shader,
        entry_point: "vs_main",
        buffers: &[QuadVertex::describe(), HitObjectModel::describe()],
        compilation_options: wgpu::PipelineCompilationOptions {
          constants: &HashMap::new().tap_mut(|x| {
            x.insert(String::from("1000"), self.config.hit_height);
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
        polygon_mode: wgpu::PolygonMode::Fill, // Others require Features::NON_FILL_POLYGON_MODE
        unclipped_depth: false,                // Requires Features::DEPTH_CLIP_CONTROL
        conservative: false,                   // Requires Features::CONSERVATIVE_RASTERIZATION
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

    let vertex_buffer_data = QuadVertex::vertices_quad(-0.5, 0.5);
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

    self.circle_texture.recreate(device, queue, format);
    self.finisher_texture.recreate(device, queue, format);
    self.circle_overlay_texture.recreate(device, queue, format);
    self.finisher_overlay_texture.recreate(device, queue, format);

    self.update_camera(queue);
  }
}
