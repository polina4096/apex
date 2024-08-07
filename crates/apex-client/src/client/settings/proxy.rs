use apex_framework::{
  event::CoreEvent,
  graphics::{
    color::Color,
    presentation::{frame_limiter::FrameLimiter, frame_sync::FrameSync},
  },
  time::time::Time,
};
use winit::event_loop::EventLoopProxy;

use crate::client::{
  audio::game_audio::GameAudio,
  event::ClientEvent,
  graphics::{FrameLimiterOptions, PresentModeOptions, RenderingBackend},
  screen::gameplay_screen::gameplay_screen::GameplayScreen,
};

use super::SettingsProxy;

pub struct ClientSettingsProxy<'a, 'window> {
  pub proxy: &'a EventLoopProxy<CoreEvent<ClientEvent>>,

  pub frame_limiter: &'a mut FrameLimiter,
  pub frame_sync: &'a mut FrameSync,
  pub gameplay_screen: &'a mut GameplayScreen,
  pub audio: &'a mut GameAudio,

  pub device: &'a wgpu::Device,
  pub queue: &'a wgpu::Queue,
  pub surface: &'a wgpu::Surface<'window>,
  pub config: &'a mut wgpu::SurfaceConfiguration,
}

impl<'a, 'window> SettingsProxy for ClientSettingsProxy<'a, 'window> {
  fn update_graphics_present_mode(&mut self, value: PresentModeOptions) {
    self.config.present_mode = value.into();
    self.proxy.send_event(CoreEvent::ReconfigureSurface).unwrap();
  }

  fn update_graphics_frame_limiter(&mut self, _value: FrameLimiterOptions) {
    // TODO: fix
    // self.proxy.send_event(CoreEvent::UpdateFrameLimiterConfiguration).unwrap();
  }

  fn update_graphics_max_frame_latency(&mut self, value: usize) {
    self.config.desired_maximum_frame_latency = value as u32;
    self.proxy.send_event(CoreEvent::ReconfigureSurface).unwrap();
  }

  fn update_graphics_macos_stutter_fix(&mut self, _value: bool) {
    // self.frame_sync.disable_external_sync();
    // self.proxy.send_event(CoreEvent::UpdateFrameLimiterConfiguration).unwrap();
  }

  fn update_graphics_rendering_backend(&mut self, _value: RenderingBackend) {
    self.proxy.send_event(CoreEvent::RecreateGraphicsContext).unwrap();
  }

  fn update_taiko_zoom(&mut self, value: f64) {
    self.gameplay_screen.taiko_renderer().set_zoom(self.device, self.queue, value);
  }

  fn update_taiko_scale(&mut self, value: f64) {
    self.gameplay_screen.taiko_renderer().set_scale(self.queue, value);
  }

  fn update_taiko_hit_position_x(&mut self, value: f32) {
    self.gameplay_screen.taiko_renderer().set_hit_position_x(self.queue, value);
  }

  fn update_taiko_hit_position_y(&mut self, value: f32) {
    self.gameplay_screen.taiko_renderer().set_hit_position_y(self.queue, value);
  }

  fn update_taiko_don_color(&mut self, value: Color) {
    self.gameplay_screen.taiko_renderer().set_don_color(self.device, value);
  }

  fn update_taiko_kat_color(&mut self, value: Color) {
    self.gameplay_screen.taiko_renderer().set_kat_color(self.device, value);
  }

  fn update_taiko_hit_animation(&mut self, value: bool) {
    self.gameplay_screen.taiko_renderer().set_hit_height(
      self.device,
      self.config.format,
      // Apparently setting it to f64::INFINITY leads to a crash, see https://github.com/gfx-rs/wgpu/issues/6082
      if value { 12.5 } else { 9999999.0 },
    );
  }

  fn update_audio_master_volume(&mut self, value: f32) {
    self.audio.set_master_volume(value);
  }

  fn update_audio_music_volume(&mut self, value: f32) {
    self.audio.set_music_volume(value);
  }

  fn update_audio_effect_volume(&mut self, value: f32) {
    self.audio.set_effect_volume(value);
  }

  fn update_gameplay_lead_in(&mut self, value: u64) {
    self.audio.lead_in = Time::from_ms(value as f64);
  }

  fn update_gameplay_lead_out(&mut self, value: u64) {
    self.audio.lead_out = Time::from_ms(value as f64);
  }

  fn update_gameplay_universal_offset(&mut self, value: i64) {
    self.audio.audio_offset = Time::from_ms(value as f64);
  }
}
