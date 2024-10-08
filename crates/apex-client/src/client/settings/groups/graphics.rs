use crate::client::{
  client::reconfigure_frame_sync,
  graphics::{FrameLimiterOptions, PresentModeOptions, RenderingBackend, WgpuBackend},
  settings::{proxy::ClientSettingsProxy, SettingsProxy},
};

use apex_framework::{
  event::CoreEvent, graphics::presentation::frame_sync::ExternalSyncError, SettingsGroup, SettingsSubgroup,
};
use macro_rules_attribute::derive;
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;

pub use super::super::ui as settings_ui;

#[derive(SettingsGroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct GraphicsSettingsGroup {
  #[custom(ui(name = "General"))]
  pub general: GraphicsGeneralSettingsSubgroup,
}

#[derive(SettingsSubgroup!, SmartDefault, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct GraphicsGeneralSettingsSubgroup {
  /// Controls the frame pacing
  #[default(Default::default())]
  #[custom(ui(name = "Frame Limiter"))]
  frame_limiter: FrameLimiterOptions,

  /// Graphics API presentation mode
  #[default(PresentModeOptions::VSync)]
  #[custom(ui(name = "Present Mode"))]
  present_mode: PresentModeOptions,

  /// Rendering backend to use
  #[default(RenderingBackend::Wgpu(WgpuBackend::Auto))]
  #[custom(ui(name = "Rendering Backend"))]
  rendering_backend: RenderingBackend,

  /// Hints the GPU how many frames to buffer
  #[default = 2]
  #[custom(ui(name = "Max Frame Latency", range = 0 ..= 5))]
  max_frame_latency: usize,

  /// Fixes massive macOS game stutter when alt-tabbing occluded window
  #[default = true]
  #[custom(ui(name = "Stutter Fix (macOS)"))]
  macos_stutter_fix: bool,
}

impl GraphicsGeneralSettingsSubgroupProxy for ClientSettingsProxy<'_, '_> {
  fn update_frame_limiter(&mut self, value: &FrameLimiterOptions) {
    reconfigure_frame_sync(self.frame_limiter, self.frame_sync, *value);
  }

  fn update_present_mode(&mut self, value: &PresentModeOptions) {
    self.config.present_mode = (*value).into();
    self.event_bus.dispatch(CoreEvent::ReconfigureSurface);
  }

  fn update_rendering_backend(&mut self, _value: &RenderingBackend) {
    self.event_bus.dispatch(CoreEvent::RecreateGraphicsContext);
  }

  fn update_max_frame_latency(&mut self, value: &usize) {
    self.config.desired_maximum_frame_latency = *value as u32;
    self.event_bus.dispatch(CoreEvent::ReconfigureSurface);
  }

  fn update_macos_stutter_fix(&mut self, value: &bool) {
    self.frame_sync.set_macos_stutter_fix(*value);

    if let Err(e) = (|| {
      self.frame_sync.disable_external_sync()?;
      self.frame_sync.enable_external_sync()?;

      Ok::<(), ExternalSyncError>(())
    })() {
      log::error!("Failed to toggle macos stutter fix: {:?}", e);
    }
  }
}
