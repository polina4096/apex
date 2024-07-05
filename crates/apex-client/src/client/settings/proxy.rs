use winit::event_loop::EventLoopProxy;

use crate::{
  client::{event::ClientEvent, screen::gameplay_screen::gameplay_screen::GameplayScreen},
  core::{event::CoreEvent, graphics::color::Color},
};

use super::{
  graphics::{FrameLimiterOptions, PresentModeOptions, RenderingBackend},
  settings::SettingsProxy,
};

pub struct ClientSettingsProxy<'a, 'window> {
  pub proxy: &'a EventLoopProxy<CoreEvent<ClientEvent>>,

  pub gameplay_screen: &'a mut GameplayScreen,

  pub device: &'a wgpu::Device,
  pub queue: &'a wgpu::Queue,
  pub surface: &'a wgpu::Surface<'window>,
  pub config: &'a mut wgpu::SurfaceConfiguration,
}

impl<'a, 'window> SettingsProxy for ClientSettingsProxy<'a, 'window> {
  fn update_graphics_present_mode(&mut self, value: PresentModeOptions) {
    self.config.present_mode = value.into();
    self.surface.configure(self.device, self.config);
  }

  // TODO: probably remove this event
  fn update_graphics_frame_limiter(&mut self, _value: FrameLimiterOptions) {
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
}
