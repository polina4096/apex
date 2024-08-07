// use bytemuck::Zeroable;
// use glam::{vec3, Quat};
// use wgpu::util::DeviceExt;

// use apex_framework::core::graphics::{bindable::Bindable, camera::{Camera2D, ProjectionOrthographic}, color::Color, graphics::Graphics, instance::Instance, layout::Layout, scene::Scene, texture::Texture, uniform::Uniform};

// use super::{model::Model, vertex::Vertex};

// pub struct TaikoRenderer {
//   pub texture : Texture,

//   pub vertex_buffer      : wgpu::Buffer,
//   pub vertex_buffer_data : Vec<Vertex>,

//   pub instance_buffer : wgpu::Buffer,
//   pub instances       : Vec<Model>,
// }

// impl TaikoRenderer {
//   pub fn new(graphics: &Graphics) -> Self {
//     // Scene
//     let scene = Scene::<ProjectionOrthographic, Camera2D> {
//       projection : ProjectionOrthographic::new(graphics.config.width, graphics.config.height, -100.0, 100.0),
//       camera     : Camera2D::new(vec3(0.0, 0.0, -50.0), Quat::zeroed(), vec3(graphics.scale as f32, graphics.scale as f32, 1.0)),
//       uniform    : Uniform::new(&graphics.device),
//     };

//     let texture_layout = graphics.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
//       entries: &[
//         wgpu::BindGroupLayoutEntry {
//           binding: 0,
//           visibility: wgpu::ShaderStages::FRAGMENT,
//           ty: wgpu::BindingType::Texture {
//             multisampled: false,
//             view_dimension: wgpu::TextureViewDimension::D2,
//             sample_type: wgpu::TextureSampleType::Float { filterable: true },
//           },
//           count: None,
//         },
//         wgpu::BindGroupLayoutEntry {
//           binding: 1,
//           visibility: wgpu::ShaderStages::FRAGMENT,
//           ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//           count: None,
//         },
//       ],
//       label: Some("default_texture_bind_group_layout"),
//     });

//     let shader = graphics.device.create_shader_module(wgpu::include_wgsl!("quad_shader.wgsl"));
//     let render_pipeline_layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
//         label: Some("Render Pipeline Layout"),
//         bind_group_layouts: &[
//           scene.layout(),
//           &texture_layout,
//         ],
//         push_constant_ranges: &[],
//     });

//     let pipeline = graphics.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//       label: Some("Render Pipeline"),
//       layout: Some(&render_pipeline_layout),
//       vertex: wgpu::VertexState {
//         module: &shader,
//         entry_point: "vs_main",
//         buffers: &[
//           Vertex::describe(),
//           Model::describe(),
//         ],
//       },
//       fragment: Some(wgpu::FragmentState {
//         module: &shader,
//         entry_point: "fs_main",
//         targets: &[Some(wgpu::ColorTargetState {
//           format: graphics.config.format,
//           blend: Some(wgpu::BlendState::ALPHA_BLENDING),
//           write_mask: wgpu::ColorWrites::ALL,
//         })],
//       }),
//       primitive: wgpu::PrimitiveState {
//         topology: wgpu::PrimitiveTopology::TriangleList,
//         strip_index_format: None,
//         front_face: wgpu::FrontFace::Ccw,
//         cull_mode: Some(wgpu::Face::Back),
//         // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
//         polygon_mode: wgpu::PolygonMode::Fill,
//         // Requires Features::DEPTH_CLIP_CONTROL
//         unclipped_depth: false,
//         // Requires Features::CONSERVATIVE_RASTERIZATION
//         conservative: false,
//       },
//       depth_stencil: None,
//       multisample: wgpu::MultisampleState {
//         count: 1,
//         mask: !0,
//         alpha_to_coverage_enabled: false,
//       },
//       multiview: None,
//     });

//     let vertex_buffer_data = Vertex::vertices_quad(-0.5, 0.5);
//     let vertex_buffer = graphics.device.create_buffer_init(
//       &wgpu::util::BufferInitDescriptor {
//         label    : Some("Vertex Buffer"),
//         contents : bytemuck::cast_slice(&vertex_buffer_data),
//         usage    : wgpu::BufferUsages::VERTEX,
//       }
//     );

//     let instances = vec![ Model { position: vec3(0.0,0.0,0.0), scale: vec3(128.0,128.0,1.0), rotation: Quat::zeroed(), color: Color::from_rgb(255, 255 ,255) } ];
//     let instance_data = instances.iter().map(Instance::bake).collect::<Vec<_>>();
//     let instance_buffer = graphics.device.create_buffer_init(
//       &wgpu::util::BufferInitDescriptor {
//         label    : Some("Instance Buffer"),
//         contents : bytemuck::cast_slice(&instance_data),
//         usage    : wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
//       }
//     );

//     let texture = Texture::from_path("/Users/polina4096/dev/apex/crates/apex-client/res/taikohitcircle.png", graphics).unwrap();

//     return Self {
//       scene,

//       pipeline,

//       texture,

//       vertex_buffer,
//       vertex_buffer_data,

//       instance_buffer,
//       instances,
//     };
//   }

//   pub fn prepare(&mut self, graphics: &Graphics) {
//     self.scene.update(&graphics.queue);
//   }

//   pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
//     rpass.set_pipeline(&self.pipeline);

//     self.scene.bind(rpass, 0);
//     rpass.set_bind_group(1, &self.texture.bind_group, &[]);

//     rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
//     rpass.set_vertex_buffer(1, self.instance_buffer.slice(..));
//     rpass.draw(0 .. self.vertex_buffer_data . len() as u32,
//                0 .. self.instances          . len() as u32);
//   }
// }
