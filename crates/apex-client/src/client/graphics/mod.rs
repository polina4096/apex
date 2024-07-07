pub mod taiko_renderer;
pub mod taiko_video_exporter;

use serde::{Deserialize, Serialize};
use wgpu::Backends;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameLimiterOptions {
  /// Software-based frame limiter
  Custom(u32),

  /// macOS-specific frame callback
  DisplayLink,

  /// Immediate presentation
  Unlimited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PresentModeOptions {
  /// Graphics API VSync
  VSync,

  /// Immediate presentation
  Immediate,
}

impl From<PresentModeOptions> for wgpu::PresentMode {
  fn from(value: PresentModeOptions) -> Self {
    match value {
      PresentModeOptions::VSync => wgpu::PresentMode::AutoVsync,
      PresentModeOptions::Immediate => wgpu::PresentMode::Immediate,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderingBackend {
  Wgpu(WgpuBackend),
  None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WgpuBackend {
  Auto,
  Vulkan,
  Metal,
  Dx12,
  Gl,
  WebGpu,
}

impl From<WgpuBackend> for Backends {
  fn from(value: WgpuBackend) -> Self {
    return match value {
      WgpuBackend::Auto => Backends::all(),
      WgpuBackend::Vulkan => Backends::VULKAN,
      WgpuBackend::Metal => Backends::METAL,
      WgpuBackend::Dx12 => Backends::DX12,
      WgpuBackend::Gl => Backends::GL,
      WgpuBackend::WebGpu => Backends::BROWSER_WEBGPU,
    };
  }
}
