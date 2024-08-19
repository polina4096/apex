use apex_framework::{
  event::CoreEvent,
  graphics::{
    framebuffer::framebuffer::Framebuffer,
    presentation::{frame_limiter::FrameLimiter, frame_sync::FrameSync},
  },
};
use winit::event_loop::EventLoopProxy;

use crate::client::{
  audio::game_audio::GameAudio, event::ClientEvent, screen::gameplay_screen::gameplay_screen::GameplayScreen,
};

use super::{
  AudioSettingsGroupProxy, GameplaySettingsGroupProxy, GraphicsSettingsGroupProxy, InterfaceSettingsGroupProxy,
  ProfileSettingsGroupProxy, SettingsProxy, TaikoSettingsGroupProxy,
};

pub struct ClientSettingsProxy<'a, 'window> {
  pub proxy: &'a EventLoopProxy<CoreEvent<ClientEvent>>,

  pub frame_limiter: &'a mut FrameLimiter,
  pub frame_sync: &'a mut FrameSync,
  pub gameplay_screen: &'a mut GameplayScreen,
  pub backbuffer: &'a mut Framebuffer,
  pub audio: &'a mut GameAudio,

  pub device: &'a wgpu::Device,
  pub queue: &'a wgpu::Queue,
  pub surface: &'a wgpu::Surface<'window>,
  pub config: &'a mut wgpu::SurfaceConfiguration,
  pub width: f32,
  pub height: f32,
}

impl SettingsProxy for ClientSettingsProxy<'_, '_> {}

impl AudioSettingsGroupProxy for ClientSettingsProxy<'_, '_> {}
impl ProfileSettingsGroupProxy for ClientSettingsProxy<'_, '_> {}
impl TaikoSettingsGroupProxy for ClientSettingsProxy<'_, '_> {}
impl InterfaceSettingsGroupProxy for ClientSettingsProxy<'_, '_> {}
impl GameplaySettingsGroupProxy for ClientSettingsProxy<'_, '_> {}
impl GraphicsSettingsGroupProxy for ClientSettingsProxy<'_, '_> {}
