use std::path::Path;

use image::{DynamicImage, GenericImageView};

use super::{bindable::Bindable, graphics::Graphics};

pub struct Texture {
  pub texture      : wgpu::Texture,
  pub bind_group   : wgpu::BindGroup,
  pub texture_view : wgpu::TextureView,
  pub sampler      : wgpu::Sampler,
}

#[allow(clippy::result_unit_err)]
impl Texture {
  // TODO: proper error handling
  pub fn from_memory(bytes: &[u8], graphics: &Graphics) -> Result<Self, ()> {
      let Ok(image) = image::load_from_memory(bytes) else { Err(())? };
      return Ok(Self::from_image(image, graphics));
  }
  pub fn from_path(path: impl AsRef<Path>, graphics: &Graphics) -> Result<Self, ()> {
      let Ok(image) = image::open(path) else { Err(())? };
      return Ok(Self::from_image(image, graphics));
  }
  pub fn from_image(image: DynamicImage, graphics: &Graphics) -> Self {
      let dimensions = image.dimensions();
      let rgba_data = image.to_rgba8();

      return Self::from_bytes(&rgba_data, dimensions, graphics);
  }
  pub fn from_bytes(data: &[u8], dimensions: (u32, u32), graphics: &Graphics) -> Self {
    let size = wgpu::Extent3d {
      width: dimensions.0,
      height: dimensions.1,
      depth_or_array_layers: 1,
    };

    // Instantiate the texture resource on the GPU side
    let texture = graphics.device.create_texture(
      &wgpu::TextureDescriptor {
        size,
        mip_level_count : 1,
        sample_count    : 1,
        dimension       : wgpu::TextureDimension::D2,
        format          : wgpu::TextureFormat::Rgba8UnormSrgb,
        usage           : wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        label           : Some("diffuse_texture"),
        view_formats    : &[],
      }
    );

    // Upload texture to GPU
    graphics.queue.write_texture(
      wgpu::ImageCopyTexture {
        texture: &texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      data,
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(4 * dimensions.0),
        rows_per_image: Some(dimensions.1),
      },
      size
    );

    // Prepare the view and sampler
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
      label: None, // TODO: use filename from path
      .. Default::default()
    });


    let sampler = graphics.device.create_sampler(&wgpu::SamplerDescriptor {
      address_mode_u : wgpu::AddressMode::ClampToEdge,
      address_mode_v : wgpu::AddressMode::ClampToEdge,
      address_mode_w : wgpu::AddressMode::ClampToEdge,
      mag_filter     : wgpu::FilterMode::Linear,
      min_filter     : wgpu::FilterMode::Nearest,
      mipmap_filter  : wgpu::FilterMode::Nearest,
      .. Default::default()
    });

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

    // Bind group
    let bind_group = graphics.device.create_bind_group(
      &wgpu::BindGroupDescriptor {
        layout: &texture_layout,
        entries: &[
          wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::TextureView(&texture_view),
          },
          wgpu::BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Sampler(&sampler),
          }
        ],
        label: Some("texture_bind_group"),
      }
    );

    return Self {
      texture,
      bind_group,
      texture_view,
      sampler,
    };
  }

  pub fn default(graphics: &Graphics) -> Self {
    return Self::from_bytes(&[0xf8, 0x06, 0x6d, 0xFF], (1, 1), graphics);
  }

  pub fn update(&mut self, graphics: &Graphics, image: DynamicImage) {
    let dimensions = image.dimensions();
    let rgba_data = image.to_rgba8();

    let size = wgpu::Extent3d {
      width: dimensions.0,
      height: dimensions.1,
      depth_or_array_layers: 1,
    };

    graphics.queue.write_texture(
      wgpu::ImageCopyTexture {
        texture: &self.texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      &rgba_data,
      wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row: Some(4 * dimensions.0),
        rows_per_image: Some(dimensions.1),
      },
      size
    );
  }
}

impl Bindable for Texture {
  fn bind<'pass, 'uniform: 'pass>(&'uniform self, render_pass: &mut wgpu::RenderPass<'pass>, index: u32) {
    render_pass.set_bind_group(index, &self.bind_group, &[]);
  }

  fn group(&self) -> &wgpu::BindGroup {
    return &self.bind_group;
  }
}
