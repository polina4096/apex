use bytemuck::Zeroable;
use glam::{vec2, vec3, vec4, Quat, Vec4};
use wgpu::util::DeviceExt;

use crate::{client::taiko::{hit_object::TaikoColor, beatmap::Beatmap}, core::{graphics::{bindable::Bindable, camera::{Camera as _, Camera2D, ProjectionOrthographic}, color::Color, graphics::Graphics, instance::Instance, layout::Layout, quad_renderer::data::quad_vertex::QuadVertex, scene::Scene, texture::Texture, uniform::Uniform}, time::{clock::AbstractClock, time::Time}}};

use super::data::hit_object_model::{BakedHitObjectModel, HitObjectModel};

pub struct TaikoRenderer {
  pub scene: Scene<ProjectionOrthographic, Camera2D>,

  pub pipeline: wgpu::RenderPipeline,

  pub time_uniform: Uniform<f32>,

  pub circle_texture           : Texture,
  pub finisher_texture         : Texture,
  pub circle_overlay_texture   : Texture,
  pub finisher_overlay_texture : Texture,

  pub vertex_buffer      : wgpu::Buffer,
  pub vertex_buffer_data : Vec<QuadVertex>,

  pub instance_buffer : wgpu::Buffer,
  pub instances       : Vec<HitObjectModel>,

  pub culling : usize,
  pub hit_idx : usize,
}

impl TaikoRenderer {
  pub fn new(graphics: &Graphics) -> Self {
    // Scene
    let scene = Scene::<ProjectionOrthographic, Camera2D> {
      projection : ProjectionOrthographic::new(graphics.config.width, graphics.config.height, -100.0, 100.0),
      camera     : Camera2D::new(vec3(0.0, 0.0, -50.0), Quat::zeroed(), vec3(graphics.scale as f32, graphics.scale as f32, 1.0)),
      uniform    : Uniform::new(&graphics.device),
    };

    let texture_layout = graphics.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

    let time_uniform = Uniform::new(&graphics.device);

    let shader = graphics.device.create_shader_module(wgpu::include_wgsl!("taiko_shader.wgsl"));
    let render_pipeline_layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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

    let pipeline = graphics.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label  : Some("Render Pipeline"),
      layout : Some(&render_pipeline_layout),

      vertex: wgpu::VertexState {
        module      : &shader,
        entry_point : "vs_main",
        buffers     : &[
          QuadVertex::describe(),
          HitObjectModel::describe(),
        ],
      },
      fragment: Some(wgpu::FragmentState {
        module      : &shader,
        entry_point : "fs_main",
        targets     : &[Some(wgpu::ColorTargetState {
          format     : graphics.config.format,
          blend      : Some(wgpu::BlendState::ALPHA_BLENDING),
          write_mask : wgpu::ColorWrites::ALL,
        })],
      }),

      primitive: wgpu::PrimitiveState {
        topology           : wgpu::PrimitiveTopology::TriangleList,
        front_face         : wgpu::FrontFace::Ccw,
        cull_mode          : Some(wgpu::Face::Back),
        polygon_mode       : wgpu::PolygonMode::Fill, // Others require Features::NON_FILL_POLYGON_MODE
        unclipped_depth    : false,                   // Requires Features::DEPTH_CLIP_CONTROL
        conservative       : false,                   // Requires Features::CONSERVATIVE_RASTERIZATION
        strip_index_format : None,
      },

      multisample: wgpu::MultisampleState {
        count                     : 1,
        mask                      : !0,
        alpha_to_coverage_enabled : false,
      },

      depth_stencil: None,
      multiview: None,
    });

    let vertex_buffer_data = QuadVertex::vertices_quad(-0.5, 0.5);
    let vertex_buffer = graphics.device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label    : Some("Vertex Buffer"),
        contents : bytemuck::cast_slice(&vertex_buffer_data),
        usage    : wgpu::BufferUsages::VERTEX,
      }
    );

    // Circle instances
    let instances = vec![];
    let instance_data = instances.iter().map(Instance::bake).collect::<Vec<_>>();
    let instance_buffer = graphics.device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label    : Some("Instance Buffer"),
        contents : bytemuck::cast_slice(&instance_data),
        usage    : wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      }
    );

    let circle_texture = Texture::from_path("/Users/polina4096/dev/apex/crates/apex-client/res/taikohitcircle.png", graphics).unwrap();
    let finisher_texture = Texture::from_path("/Users/polina4096/dev/apex/crates/apex-client/res/taikobigcircle.png", graphics).unwrap();
    let circle_overlay_texture = Texture::from_path("/Users/polina4096/dev/apex/crates/apex-client/res/taikohitcircleoverlay.png", graphics).unwrap();
    let finisher_overlaytexture = Texture::from_path("/Users/polina4096/dev/apex/crates/apex-client/res/taikobigcircleoverlay.png", graphics).unwrap();

    return Self {
      scene,

      pipeline,

      time_uniform,

      circle_texture,
      finisher_texture,
      circle_overlay_texture,
      finisher_overlay_texture: finisher_overlaytexture,

      vertex_buffer,
      vertex_buffer_data,

      instance_buffer,
      instances,

      culling: 0,
      hit_idx: 0,
    };
  }

  pub fn prepare(&mut self, graphics: &Graphics, beatmap: &Beatmap, clock: &mut impl AbstractClock) {
    let zoom = 0.235;
    let scale = 0.85;
    let hit_pos = vec2(300.0, 300.0);
    let audio_offset = 0.0;

    let audio_offset = Time::from_seconds(audio_offset / 1000.0);
    let time = clock.position();

    while let Some(circle) = beatmap.hit_objects.get(self.hit_idx) {
      // When you snap to a certain object on a timeline, this thing counts it as being hit
      // In order to render the object if (obj.time == current_time), we offset it by a bit
      // TODO: this is most certainly not the best way to handle this, but whatever

      let tolerance = Time::from_ms(2);
      if circle.time + audio_offset + tolerance <= time {
        self.hit_idx += 1;

        let len = self.instances.len();
        let idx = len - self.hit_idx;
        let instance = &mut self.instances[idx];
        instance.hit = time * 1000.0 * zoom;

        let single_baked = [instance.bake()];
        let byte_slice: &[u8] = bytemuck::cast_slice(&single_baked);
        let offset = std::mem::size_of::<BakedHitObjectModel>() * idx;

        graphics.queue.write_buffer(&self.instance_buffer, offset as wgpu::BufferAddress, byte_slice);
      } else { break }
    }

    // Update scene matrix
    let scale = (graphics.scale * scale) as f32;
    self.scene.camera.set_scale(vec3(scale, scale, 1.0));
    self.scene.camera.set_x((hit_pos.x / scale) as f32);
    self.scene.camera.set_y((hit_pos.y / scale) as f32);
    self.scene.update(&graphics.queue);


    // Update time uniform
    let time_offset = (audio_offset - time).to_seconds() * 1000.0 * zoom;
    self.time_uniform.update(&graphics.queue, &(time_offset as f32));
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
    rpass.draw(0 ..  self.vertex_buffer_data.len()        as u32,
               0 .. (self.instances.len() - self.culling) as u32);
  }

  pub fn reset_instances(&mut self, graphics: &Graphics) {
    for inst in &mut self.instances {
      inst.hit = Time::zero();
    }

    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    let byte_slice = bytemuck::cast_slice(&instance_data);
    let offset = 0 as wgpu::BufferAddress;

    graphics.queue.write_buffer(&self.instance_buffer, offset, byte_slice);
  }

  pub fn prepare_instances(&mut self, graphics: &Graphics, beatmap: &Beatmap) {
    const CIRCLE_SIZE: f32 = 128.0;

    let zoom = 0.235;
    let don_color = Color::new(0.973, 0.596, 0.651, 1.0);
    let kat_color = Color::new(0.741, 0.698, 0.827, 1.0);

    self.instances.clear();

    // Taiko
    const OSU_TAIKO_VELOCITY_MULTIPLIER: f64 = 1.4;

    let mut idx_t = beatmap.timing_points.len() - 1;
    let mut idx_v = beatmap.velocity_points.len() - 1;
    for obj in beatmap.hit_objects.iter().rev() {
      while beatmap.timing_points[idx_t].time > obj.time && idx_t != 0 { idx_t -= 1; }
      while beatmap.velocity_points[idx_v].time > obj.time && idx_v != 0 { idx_v -= 1; }

      // Timing
      let beat_length = 60.0 / beatmap.timing_points[idx_t].bpm * 1000.0; // we want ms...
      let velocity = beatmap.velocity_points[idx_v].velocity;

      let base_length = 1000.0;
      let multiplier = OSU_TAIKO_VELOCITY_MULTIPLIER * velocity * base_length / beat_length * beatmap.velocity_multiplier as f64;

      self.instances.push(HitObjectModel {
        time: (obj.time.to_seconds() * 1000.0 * zoom) as f32,
        size: if obj.big { vec2(CIRCLE_SIZE * 1.55, CIRCLE_SIZE * 1.55) }
              else       { vec2(CIRCLE_SIZE       , CIRCLE_SIZE       ) },

        color: if obj.color == TaikoColor::KAT { kat_color }  // vec4(0.0, 0.47, 0.67, 1.0)
               else                            { don_color }, // vec4(0.92, 0.0, 0.27, 1.0)

        finisher: obj.big,
        hit: Time::zero(),
        velocity: multiplier as f32,
      });
    }

    let instance_data = self.instances.iter().map(Instance::bake).collect::<Vec<_>>();
    self.instance_buffer = graphics.device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label    : Some("Instance Buffer"),
        contents : bytemuck::cast_slice(&instance_data),
        usage    : wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
      }
    );
  }
}
