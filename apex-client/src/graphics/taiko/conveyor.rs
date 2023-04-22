use cgmath::{Quaternion, vec3, Zero, vec2, Vector4, vec4};
use wcore::graphics::{texture::Texture, scene::Scene, camera::{ProjectionOrthographic, Camera2D, Camera}, uniform::Uniform, common::{vertex::Vertex, model::Model}, context::Graphics, instance::Instance, bindable::Bindable};
use wgpu::util::DeviceExt;

use crate::{layer::taiko::TaikoState, taiko::{parser::Beatmap, taiko_circle::TaikoColor}};

use super::model::TaikoHitObjectModel;

const CIRCLE_SIZE: f32 = 128.0;

pub struct Conveyor {
    pub t_hitcircle  : Texture,
    pub t_bigcircle  : Texture,
    pub t_hitoverlay : Texture,
    pub t_bigoverlay : Texture,
    pub t_hitposition : Texture,

    pub scene           : Scene<ProjectionOrthographic, Camera2D>,
    pub time_uniform    : Uniform<Vector4<f32>>,
    pub circle_pipeline : wgpu::RenderPipeline,
    pub hitpos_pipeline : wgpu::RenderPipeline,

    pub vertex_buffer      : wgpu::Buffer,
    pub vertex_buffer_data : Vec<Vertex>,

    pub circle_instance_buffer : wgpu::Buffer,
    pub circle_instances       : Vec<TaikoHitObjectModel>,
    
    pub hitpos_instance_buffer : wgpu::Buffer,
    pub hitpos_instances       : Vec<Model>,

    // TOOD: reset to 0 on map load to prevent crash
    pub cull_back : usize,
}

impl Conveyor {
    pub fn new(graphics: &Graphics) -> Self {
        // Texture
        let hitcircle = Texture::from_memory(include_bytes!("../../../res/taikohitcircle.png"), graphics).expect("Failed to load texture");
        let bigcircle = Texture::from_memory(include_bytes!("../../../res/taikobigcircle.png"), graphics).expect("Failed to load texture");
        let hitoverlay = Texture::from_memory(include_bytes!("../../../res/taikohitcircleoverlay.png"), graphics).expect("Failed to load texture");
        let bigoverlay = Texture::from_memory(include_bytes!("../../../res/taikobigcircleoverlay.png"), graphics).expect("Failed to load texture");
        let hitposition = Texture::from_memory(include_bytes!("../../../res/approachcircle.png"), graphics).expect("Failed to load texture");

        // Vertices
        let vertex_buffer_data = Vertex::vertices_quad(-0.5, 0.5);
        let vertex_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label    : Some("Vertex Buffer"),
                contents : bytemuck::cast_slice(&vertex_buffer_data),
                usage    : wgpu::BufferUsages::VERTEX,
            }
        );

        // Hit position instances
        let hitpos_instances = vec![ Model { position: vec3(0.0, 0.0, 0.0), scale: vec3(128.0, 128.0, 1.0), rotation: Quaternion::zero() } ];
        let hitpos_instance_data = hitpos_instances.iter().map(Instance::bake).collect::<Vec<_>>();
        let hitpos_instance_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label    : Some("Instance Buffer"),
                contents : bytemuck::cast_slice(&hitpos_instance_data),
                usage    : wgpu::BufferUsages::VERTEX,
            }
        );

        // Circle instances
        let circle_instances = vec![];
        let circle_instance_data = circle_instances.iter().map(Instance::bake).collect::<Vec<_>>();
        let circle_instance_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label    : Some("Instance Buffer"),
                contents : bytemuck::cast_slice(&circle_instance_data),
                usage    : wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        // Scene
        let scene = Scene::<ProjectionOrthographic, Camera2D> {
            projection : ProjectionOrthographic::new(graphics.config.width, graphics.config.height, -100.0, 100.0),
            camera     : Camera2D::new(vec3(0.0, 0.0, -50.0), Quaternion::zero(), vec3(graphics.scale as f32, graphics.scale as f32, 1.0)),
            uniform    : Uniform::new(&graphics.device),
        };

        // Time uniform
        let time_uniform = Uniform::new(&graphics.device);

        // Hitpos pipeline
        let shader = graphics.device.create_shader_module(wgpu::include_wgsl!("../../../res/common.wgsl"));
        let render_pipeline_layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                scene       . layout(),
                hitposition . layout(),
            ],
            push_constant_ranges: &[],
        });

        let hitpos_pipeline = graphics.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    Vertex::describe(),
                    Model::describe(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: graphics.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
        });

        // Circle pipeline
        let shader = graphics.device.create_shader_module(wgpu::include_wgsl!("../../../res/taiko.wgsl"));
        let render_pipeline_layout = graphics.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                scene       . layout(),
                time_uniform. layout(),
                hitcircle   . layout(),
                hitoverlay  . layout(),
                bigcircle   . layout(),
                bigoverlay  . layout(),
            ],
            push_constant_ranges: &[],
        });

        let circle_pipeline = graphics.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label  : Some("Render Pipeline"),
            layout : Some(&render_pipeline_layout),

            vertex: wgpu::VertexState {
                module      : &shader,
                entry_point : "vs_main",
                buffers     : &[
                    Vertex::describe(),
                    TaikoHitObjectModel::describe(),
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

        return Self {
            t_hitcircle   : hitcircle,
            t_bigcircle   : bigcircle,
            t_hitoverlay  : hitoverlay,
            t_bigoverlay  : bigoverlay,
            t_hitposition : hitposition,

            scene,
            time_uniform,
            circle_pipeline,
            hitpos_pipeline,

            vertex_buffer,
            vertex_buffer_data,
            
            circle_instance_buffer,
            circle_instances,
            hitpos_instance_buffer,
            hitpos_instances,

            cull_back: 0,
        };
    }

    pub fn draw<'a: 'b, 'b>(&'a mut self, rebuild_instances: bool, state: &TaikoState, beatmap: &Beatmap, time_ms: u32, render_pass: &mut wgpu::RenderPass<'b>, graphics: &mut Graphics) {
        if rebuild_instances { self.rebuild_instances_beatmap(state, beatmap, graphics); }

        // Circle culling
        if state.hit_circles {
            while let Some(circle) = beatmap.objects.get(self.cull_back) {
                if circle.time.to_ms() as i64 + state.audio_offset <= time_ms as i64 {
                    self.cull_back += 1;
                } else { break }
            }
        }

        // Update scene matrix
        let scale = graphics.scale as f32 * state.scale;
        self.scene.camera.set_scale(vec3(scale, scale, 1.0));
        self.scene.camera.set_x(state.hit_position.x / scale);
        self.scene.camera.set_y(state.hit_position.y / scale);
        self.scene.update(&graphics.queue);
        
        // Update time matrix
        let time_offset = (- (time_ms as f32) + state.audio_offset as f32) * state.zoom;
        self.time_uniform.update(&graphics.queue, &vec4(time_offset, 0.0, 0.0, 0.0));
        
        // Hit position
        render_pass.set_pipeline(&self.hitpos_pipeline);

        self.scene.bind(render_pass, 0);
        render_pass.set_bind_group(1, &self.t_hitposition.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.hitpos_instance_buffer.slice(..));
        render_pass.draw(0 .. self.vertex_buffer_data.len() as u32, 
                         0 .. self.hitpos_instances.len() as u32);

        // Circles
        render_pass.set_pipeline(&self.circle_pipeline);
        
        self.scene.bind(render_pass, 0);
        self.time_uniform.bind(render_pass, 1);
        render_pass.set_bind_group(2, &self.t_hitcircle.bind_group, &[]);
        render_pass.set_bind_group(3, &self.t_hitoverlay.bind_group, &[]);
        render_pass.set_bind_group(4, &self.t_bigcircle.bind_group, &[]);
        render_pass.set_bind_group(5, &self.t_bigoverlay.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.circle_instance_buffer.slice(..));
        render_pass.draw(0 ..  self.vertex_buffer_data . len()                   as u32, 
                         0 .. (self.circle_instances   . len() - self.cull_back) as u32);
    }

    fn rebuild_instances_beatmap(&mut self, state: &TaikoState, beatmap: &Beatmap, graphics: &Graphics) {
        self.circle_instances.clear();
        
        let mut idx_t = beatmap.timing.len() - 1;
        let mut idx_v = beatmap.velocity.len() - 1;
        for obj in beatmap.objects.iter().rev() {
            while beatmap.timing[idx_t].time > obj.time && idx_t != 0 { idx_t -= 1; }
            while beatmap.velocity[idx_v].time > obj.time && idx_v != 0 { idx_v -= 1; }
            
            // Taiko
            const OSU_TAIKO_VELOCITY_MULTIPLIER: f64 = 1.4;

            // Timing            
            let beat_length = 60.0 / beatmap.timing[idx_t].bpm * 1000.0; // we want ms...
            let velocity = beatmap.velocity[idx_v].velocity;

            let base_length = 1000.0;
            let multiplier = OSU_TAIKO_VELOCITY_MULTIPLIER * velocity * base_length / beat_length * beatmap.velocity_multiplier as f64;

            self.circle_instances.push(TaikoHitObjectModel {
                time: obj.time.to_ms() as f32 * state.zoom,
                size: if obj.big { vec2(CIRCLE_SIZE * 1.55, CIRCLE_SIZE * 1.55) }
                      else       { vec2(CIRCLE_SIZE       , CIRCLE_SIZE       ) },

                color: if obj.color == TaikoColor::KAT { state.kat_color }  // vec4(0.0, 0.47, 0.67, 1.0)
                       else                            { state.don_color }, // vec4(0.92, 0.0, 0.27, 1.0)
                
                finisher: obj.big,
                velocity: multiplier as f32,
            });
        }

        let instance_data = self.circle_instances.iter().map(Instance::bake).collect::<Vec<_>>();
        self.circle_instance_buffer = graphics.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label    : Some("Instance Buffer"),
                contents : bytemuck::cast_slice(&instance_data),
                usage    : wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
    }

}