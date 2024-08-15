pub trait Drawable {
  fn recreate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat);

  fn resize(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, width: f32, height: f32);
  fn resize_width(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f32);
  fn resize_height(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f32);
  fn rescale(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, value: f32);
}
