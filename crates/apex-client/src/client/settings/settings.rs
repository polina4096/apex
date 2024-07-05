use super::graphics::{FrameLimiterOptions, PresentModeOptions, RenderingBackend, WgpuBackend};
use log::error;
use serde::{Deserialize, Serialize};

use crate::{core::graphics::color::Color, settings};

settings! {
  audio {
    /// Master volume
    master_volume: f32 = 0.25

    /// Music volume
    music_volume: f32 = 1.0

    /// Effect volume
    effect_volume: f32 = 1.0

    /// TODO: remove after implementing volumes
    hitsounds: bool = true
  }

  graphics {
    /// Controls the frame pacing
    frame_limiter: FrameLimiterOptions = {
      if cfg!(target_os = "macos") {
        FrameLimiterOptions::DisplayLink
      } else {
        FrameLimiterOptions::Unlimited
      }
    }

    /// Graphics API presentation mode
    present_mode: PresentModeOptions = PresentModeOptions::VSync

    /// Rendering backend to use
    rendering_backend: RenderingBackend = RenderingBackend::Wgpu(WgpuBackend::Auto)

  }

  gameplay {
    /// Offset of the audio in milliseconds
    universal_offset: i64 = 0

    /// Additional time before the first note
    lead_in: u64 = 1000

    /// Additional time after the last note
    lead_out: u64 = 1000
  }

  taiko {
    /// Hit object distance multiplier
    zoom: f64 = 0.235

    /// Gameplay scale
    scale: f64 = 0.85

    /// Hit position X
    hit_position_x: f32 = 256.0

    /// Hit position Y
    hit_position_y: f32 = 192.0

    /// Color of the don hit object
    don_color: Color = Color::new(0.92, 0.00, 0.27, 1.00)

    /// Color of the kat hit object
    kat_color: Color = Color::new(0.00, 0.47, 0.67, 1.00)
  }
}

impl Drop for Settings {
  fn drop(&mut self) {
    let string = match toml::to_string_pretty(self) {
      Ok(string) => string,
      Err(e) => {
        error!("Failed to serialize config: {}", e);
        return;
      }
    };

    if let Err(e) = std::fs::write("./config.toml", string) {
      error!("Failed to write config to disk: {}", e);
    }
  }
}
