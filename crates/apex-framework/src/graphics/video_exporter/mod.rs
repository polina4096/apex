use std::{
  fmt::Display,
  io::Write as _,
  path::PathBuf,
  process::{Command, Stdio},
};

use log::error;

pub trait VideoExporterCallback {
  type Data;
  fn prepare_frame(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: Self::Data);

  fn render_frame<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>);
}

#[derive(Debug, Clone)]
pub struct DisplayMode {
  pub width: u32,
  pub height: u32,
  pub framerate: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingPreset {
  Ultrafast,
  Superfast,
  Veryfast,
  Faster,
  Fast,
  Medium,
  Slow,
  Slower,
  Veryslow,
  Placebo,
}

impl Display for EncodingPreset {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Ultrafast => write!(f, "ultrafast"),
      Self::Superfast => write!(f, "superfast"),
      Self::Veryfast => write!(f, "veryfast"),
      Self::Faster => write!(f, "faster"),
      Self::Fast => write!(f, "fast"),
      Self::Medium => write!(f, "medium"),
      Self::Slow => write!(f, "slow"),
      Self::Slower => write!(f, "slower"),
      Self::Veryslow => write!(f, "veryslow"),
      Self::Placebo => write!(f, "placebo"),
    }
  }
}

#[derive(Debug, Clone)]
pub struct VideoExporterConfig {
  pub output_path: String,
  pub display_mode: DisplayMode,
  pub crf_bitrate: u32,
  pub preset: EncodingPreset,
}

impl Default for VideoExporterConfig {
  fn default() -> Self {
    return Self {
      output_path: String::from("/Users/polina4096/Desktop/output.mp4"),
      display_mode: DisplayMode {
        width: 1920,
        height: 1080,
        framerate: 120,
      },
      crf_bitrate: 15,
      preset: EncodingPreset::Veryfast,
    };
  }
}

pub struct VideoExporter {
  view: wgpu::TextureView,
  texture: wgpu::Texture,
  output_buffer: wgpu::Buffer,
}

impl VideoExporter {
  pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, cfg: &VideoExporterConfig) -> Self {
    // Texture View
    let texture_desc = wgpu::TextureDescriptor {
      size: wgpu::Extent3d {
        width: cfg.display_mode.width,
        height: cfg.display_mode.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: format,
      usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
      label: None,
      view_formats: &[],
    };

    let texture = device.create_texture(&texture_desc);
    let view = texture.create_view(&Default::default());

    // Output Buffer
    let u32_size = std::mem::size_of::<u32>() as u32;

    let output_buffer_size = (u32_size * cfg.display_mode.width * cfg.display_mode.height) as wgpu::BufferAddress;
    let output_buffer_desc = wgpu::BufferDescriptor {
      size: output_buffer_size,
      usage: wgpu::BufferUsages::COPY_DST
      // this tells wpgu that we want to read this buffer from the cpu
      | wgpu::BufferUsages::MAP_READ,
      label: None,
      mapped_at_creation: false,
    };

    let output_buffer = device.create_buffer(&output_buffer_desc);

    return Self { view, texture, output_buffer };
  }

  // TODO: refactor
  #[allow(clippy::too_many_arguments)]
  pub fn export<I, C>(
    &self,
    cfg: &VideoExporterConfig,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    audio: PathBuf,
    offset: u64,
    data: I,
    mut callback: C,
  ) where
    I: Iterator<Item = C::Data>,
    C: VideoExporterCallback,
  {
    let (tx_data, rx_data) = std::sync::mpsc::channel::<Vec<u8>>();

    let output = cfg.output_path.clone();
    let framerate = cfg.display_mode.framerate.to_string();
    let size = format!("{}x{}", cfg.display_mode.width, cfg.display_mode.height);
    let preset = cfg.preset.to_string();
    let crf = cfg.crf_bitrate.to_string();

    std::thread::spawn(move || {
      // do ffmpeg stuff
      let ffmpeg_cmd = "ffmpeg";

      #[rustfmt::skip]
      let args = [
        "-r".to_string(), framerate,
        "-f".to_string(), "rawvideo".to_string(),
        "-vcodec".to_string(), "rawvideo".to_string(),
        "-pix_fmt".to_string(), "bgra".to_string(),
        "-s".to_string(), size,
        "-i".to_string(), "-".to_string(),
        "-ss".to_string(), format!("{}ms", offset),
        "-i".to_string(), audio.to_str().unwrap().to_owned(),
        "-c:a".to_string(), "aac".to_string(),
        "-c:v".to_string(), "libx264".to_string(),
        "-crf".to_string(), crf,
        "-preset".to_string(), preset,
        "-pix_fmt".to_string(), "yuva420p".to_string(),
        // "-vf".to_string(), "crop=2048:1024:0:0".to_string(),
        "-shortest".to_string(),
        "-y".to_string(),
        output,
      ];

      let mut child = match Command::new(ffmpeg_cmd) //
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
      {
        Ok(child) => child,
        Err(e) => {
          error!("Failed to spawn ffmpeg process: {:?}", e);
          return;
        }
      };

      let stdin = child.stdin.as_mut().expect("Failed to open ffmpeg stdin.");

      for data in rx_data {
        stdin.write_all(data.as_slice()).unwrap();
      }

      let output = child.wait_with_output().unwrap();
      // Check the output and error messages
      if output.status.success() {
        println!("FFmpeg command executed successfully.");
      } else {
        // The `stderr` field of the output contains any error messages
        let error_message = String::from_utf8_lossy(&output.stderr);
        println!("FFmpeg command failed: {}", error_message);
      }
    });

    for data in data {
      callback.prepare_frame(device, queue, data);
      self.render_frame(device, queue, cfg, &mut callback);

      {
        let buffer_slice = self.output_buffer.slice(..);

        let (tx, rx) = std::sync::mpsc::channel();
        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
          tx.send(result).unwrap();
        });

        device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().unwrap();

        let buffer_data = buffer_slice.get_mapped_range();
        tx_data.send(buffer_data.to_vec()).unwrap();
      }

      self.output_buffer.unmap();
    }
  }
}

impl VideoExporter {
  fn render_frame(
    &self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cfg: &VideoExporterConfig,
    callback: &mut impl VideoExporterCallback,
  ) {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
      let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("egui render pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: &self.view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 }),
            store: wgpu::StoreOp::Store,
          },
        })],
        timestamp_writes: None,
        occlusion_query_set: None,
        depth_stencil_attachment: None,
      });

      callback.render_frame(&mut rpass);
    }

    // TODO: remove hardcoded video size
    let u32_size = std::mem::size_of::<u32>() as u32;

    encoder.copy_texture_to_buffer(
      wgpu::ImageCopyTexture {
        aspect: wgpu::TextureAspect::All,
        texture: &self.texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
      },
      wgpu::ImageCopyBuffer {
        buffer: &self.output_buffer,
        layout: wgpu::ImageDataLayout {
          offset: 0,
          bytes_per_row: Some(u32_size * cfg.display_mode.width),
          rows_per_image: Some(cfg.display_mode.height),
        },
      },
      wgpu::Extent3d {
        width: cfg.display_mode.width,
        height: cfg.display_mode.height,
        depth_or_array_layers: 1,
      },
    );

    queue.submit(Some(encoder.finish()));
  }
}
