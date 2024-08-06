use std::sync::Arc;

use image_loader::BackgroundImageLoader;
use wgpu::rwh::HasDisplayHandle;
use winit::window::Window;

use super::graphics::Graphics;

pub mod image_loader;

pub struct Egui {
  ctx: egui::Context,
  renderer: egui_wgpu::Renderer,
  screen_desc: egui_wgpu::ScreenDescriptor,
  winit_state: egui_winit::State,

  clipped_primitives: Vec<egui::ClippedPrimitive>,
  commands: Vec<wgpu::CommandBuffer>,
}

impl Egui {
  pub fn new(display_handle: &impl HasDisplayHandle, graphics: &Graphics) -> Self {
    let ctx = egui::Context::default();
    let renderer = egui_wgpu::Renderer::new(&graphics.device, graphics.format, None, 1, false);
    let screen_desc = egui_wgpu::ScreenDescriptor {
      size_in_pixels: [graphics.size.width, graphics.size.height],
      pixels_per_point: graphics.scale as f32,
    };

    let winit_state = egui_winit::State::new(
      ctx.clone(),
      egui::ViewportId::default(),
      display_handle,
      Some(graphics.scale as f32),
      Some(graphics.device.limits().max_texture_dimension_2d as usize),
    );

    egui_extras::install_image_loaders(&ctx);
    ctx.add_image_loader(Arc::new(BackgroundImageLoader::new(ctx.clone())));

    return Self {
      ctx,
      renderer,
      screen_desc,
      winit_state,

      clipped_primitives: Vec::new(),
      commands: Vec::new(),
    };
  }

  /// Don't clone the context as it can get invalidated!
  ///
  /// Unfortunatly, I haven't really figured out how to reset textures in egui,
  /// which means that when we recreate the context it gets completely invalidated.
  pub fn ctx(&self) -> &egui::Context {
    return &self.ctx;
  }

  pub fn renderer(&self) -> &egui_wgpu::Renderer {
    return &self.renderer;
  }

  pub fn renderer_mut(&mut self) -> &mut egui_wgpu::Renderer {
    return &mut self.renderer;
  }

  pub fn ctx_renderer_mut(&mut self) -> (&egui::Context, &mut egui_wgpu::Renderer) {
    return (&self.ctx, &mut self.renderer);
  }

  pub fn handle_window_event(
    &mut self,
    window: &Window,
    event: &winit::event::WindowEvent,
  ) -> egui_winit::EventResponse {
    return self.winit_state.on_window_event(window, event);
  }

  pub fn begin_frame(&mut self, window: &Window) {
    let new_input = self.winit_state.take_egui_input(window);
    self.ctx.begin_frame(new_input);
  }

  pub fn end_frame(&mut self, window: &Window, graphics: &Graphics, encoder: &mut wgpu::CommandEncoder) {
    let egui_output = self.ctx.end_frame();

    // Handle platform interactions
    self.winit_state.handle_platform_output(window, egui_output.platform_output);

    // Free textures
    for id in &egui_output.textures_delta.free {
      self.renderer.free_texture(id);
    }

    // Upload textures
    for (id, image_delta) in &egui_output.textures_delta.set {
      self.renderer.update_texture(&graphics.device, &graphics.queue, *id, image_delta);
    }

    // Generate vertices and render commands
    #[rustfmt::skip] let clipped_primitives = self.ctx.tessellate(egui_output.shapes, egui_output.pixels_per_point);
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

  pub fn recreate_context(&mut self, display_handle: &impl HasDisplayHandle, graphics: &Graphics) {
    *self = Self::new(display_handle, graphics);
  }
}
