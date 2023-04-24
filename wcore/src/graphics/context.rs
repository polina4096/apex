use egui_winit::winit::dpi::PhysicalSize;

pub struct Graphics {
    pub device  : wgpu::Device,
    pub surface : wgpu::Surface,
    pub queue   : wgpu::Queue,
    pub format  : wgpu::TextureFormat,
    pub config  : wgpu::SurfaceConfiguration,

    pub size    : PhysicalSize<u32>,
    pub scale   : f64,

    pub layout  : Layout,
}

pub struct Layout {
    pub texture : wgpu::BindGroupLayout,
}

impl Layout {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

        return Self {
            texture,
        };
    }
}