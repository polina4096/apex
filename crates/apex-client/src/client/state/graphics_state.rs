use serde::{Deserialize, Serialize};
use wgpu::Backends;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameLimiter {
  /// Attempt to automatically determine the best frame limiter settings for the current platform
  #[default]
  Auto,

  /// Vertical synchronization provided by the rendering backend
  VSync,

  /// macOS-specific vertical synchronization
  DisplayLink,

  /// Software-based frame limiter
  Limited(u32),

  /// No frame limiter, immediate presentation
  Unlimited,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsState {
  /// Controls the frame pacing
  pub frame_limiter: FrameLimiter,

  /// Rendering backend to use
  pub rendering_backend: RenderingBackend,
}

impl Default for GraphicsState {
  fn default() -> Self {
    return Self {
      frame_limiter: FrameLimiter::Auto,
      rendering_backend: RenderingBackend::Wgpu(WgpuBackend::Auto),
    };
  }
}
