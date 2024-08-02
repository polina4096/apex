use winit::event_loop::EventLoopProxy;

use crate::{
  client::{
    event::ClientEvent,
    graphics::{FrameLimiterOptions, PresentModeOptions, RenderingBackend},
    screen::gameplay_screen::gameplay_screen::GameplayScreen,
  },
  core::{audio::audio_mixer::AudioController, event::CoreEvent, graphics::color::Color, time::time::Time},
};

use super::SettingsProxy;

pub struct ClientSettingsProxy<'a, 'window> {
  pub proxy: &'a EventLoopProxy<CoreEvent<ClientEvent>>,

  pub gameplay_screen: &'a mut GameplayScreen,
  pub audio_controller: &'a mut AudioController,

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

  // TODO: probably remove this event
  fn update_graphics_frame_limiter(&mut self, _value: FrameLimiterOptions) {
    self.proxy.send_event(CoreEvent::UpdateFrameLimiterConfiguration).unwrap();
  }

  fn update_graphics_macos_stutter_fix(&mut self, _value: bool) {
    self.proxy.send_event(CoreEvent::UpdateFrameLimiterConfiguration).unwrap();
  }

  // TODO: probably remove this event too
  fn update_graphics_rendering_backend(&mut self, _value: RenderingBackend) {
    self.proxy.send_event(CoreEvent::RecreateGraphicsContext).unwrap();
  }

  fn update_taiko_zoom(&mut self, value: f64) {
    self.gameplay_screen.set_taiko_zoom(self.device, self.queue, value);
  }

  fn update_taiko_scale(&mut self, value: f64) {
    self.gameplay_screen.set_taiko_scale(self.queue, value);
  }

  fn update_taiko_hit_position_x(&mut self, value: f32) {
    self.gameplay_screen.set_taiko_hit_position_x(self.queue, value);
  }

  fn update_taiko_hit_position_y(&mut self, value: f32) {
    self.gameplay_screen.set_taiko_hit_position_y(self.queue, value);
  }

  fn update_taiko_don_color(&mut self, value: Color) {
    self.gameplay_screen.set_taiko_don_color(self.device, value);
  }

  fn update_taiko_kat_color(&mut self, value: Color) {
    self.gameplay_screen.set_taiko_kat_color(self.device, value);
  }

  fn update_taiko_hit_animation(&mut self, value: bool) {
    self.gameplay_screen.set_taiko_hit_animation(self.device, self.config.format, value)
  }

  fn update_audio_master_volume(&mut self, value: f32) {
    self.audio_controller.set_master_volume(value);
  }

  fn update_audio_music_volume(&mut self, value: f32) {
    self.audio_controller.set_music_volume(value);
  }

  fn update_audio_effect_volume(&mut self, value: f32) {
    self.audio_controller.set_effect_volume(value);
  }

  fn update_gameplay_lead_in(&mut self, value: u64) {
    self.gameplay_screen.set_audio_lead_in(Time::from_ms(value as f64));
  }

  fn update_gameplay_lead_out(&mut self, value: u64) {
    self.gameplay_screen.set_audio_lead_out(Time::from_ms(value as f64));
  }

  fn update_gameplay_universal_offset(&mut self, value: i64) {
    self.gameplay_screen.set_audio_offset(Time::from_ms(value as f64));
  }
}
