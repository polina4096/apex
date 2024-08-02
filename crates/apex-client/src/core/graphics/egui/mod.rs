use std::sync::Arc;

use egui::ClippedPrimitive;
use image_loader::BackgroundImageLoader;
use wgpu::rwh::HasDisplayHandle;
use winit::window::Window;

use super::graphics::Graphics;

pub mod image_loader;

pub struct EguiContext {
  pub renderer: egui_wgpu::Renderer,
  pub screen_desc: egui_wgpu::ScreenDescriptor,
  pub winit_state: egui_winit::State,
  pub image_loader: Arc<BackgroundImageLoader>,

  pub clipped_primitives: Vec<ClippedPrimitive>,
  pub commands: Vec<wgpu::CommandBuffer>,
}

impl EguiContext {
  pub fn new(display_handle: &impl HasDisplayHandle, graphics: &Graphics) -> Self {
    let context = egui::Context::default();
    let renderer = egui_wgpu::Renderer::new(&graphics.device, graphics.format, None, 1, false);
    let screen_desc = egui_wgpu::ScreenDescriptor {
      size_in_pixels: [graphics.size.width, graphics.size.height],
      pixels_per_point: graphics.scale as f32,
    };

    let winit_state = egui_winit::State::new(
      context,
      egui::ViewportId::default(),
      display_handle,
      Some(graphics.scale as f32),
      Some(graphics.device.limits().max_texture_dimension_2d as usize),
    );

    let ctx = winit_state.egui_ctx();
    egui_extras::install_image_loaders(ctx);
    let image_loader = Arc::new(BackgroundImageLoader::new(ctx.clone()));
    ctx.add_image_loader(image_loader.clone());

    return Self {
      renderer,
      screen_desc,
      winit_state,
      image_loader,

      clipped_primitives: Vec::new(),
      commands: Vec::new(),
    };
  }

  pub fn egui_ctx(&self) -> &egui::Context {
    return self.winit_state.egui_ctx();
  }

  pub fn begin_frame(&mut self, window: &Window) {
    let new_input = self.winit_state.take_egui_input(window);
    self.winit_state.egui_ctx().begin_frame(new_input);
  }

  pub fn end_frame(&mut self, graphics: &Graphics, encoder: &mut wgpu::CommandEncoder) {
    let egui_output = self.winit_state.egui_ctx().end_frame();

    // Free textures
    for id in &egui_output.textures_delta.free {
      self.renderer.free_texture(id);
    }

    // Upload textures
    for (id, image_delta) in &egui_output.textures_delta.set {
      self.renderer.update_texture(&graphics.device, &graphics.queue, *id, image_delta);
    }

    // Generate vertices and render commands
    #[rustfmt::skip] let clipped_primitives = self.winit_state.egui_ctx().tessellate(egui_output.shapes, self.screen_desc.pixels_per_point);
    #[rustfmt::skip] let commands = self.renderer.update_buffers(&graphics.device, &graphics.queue, encoder, &clipped_primitives, &self.screen_desc);

    self.clipped_primitives = clipped_primitives;
    self.commands = commands;
  }

  pub fn render<'this: 'pass, 'pass>(&'this mut self, graphics: &Graphics, render_pass: &mut wgpu::RenderPass<'pass>) {
    self.renderer.render(render_pass, &self.clipped_primitives, &self.screen_desc);
    graphics.queue.submit(std::mem::take(&mut self.commands));
  }

  pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
    self.screen_desc.size_in_pixels = [new_size.width, new_size.height];
  }

  pub fn scale(&mut self, scale_factor: f64) {
    self.screen_desc.pixels_per_point = scale_factor as f32;
  }
}
