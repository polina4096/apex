pub trait Layout {
  fn layout(&self) -> &wgpu::BindGroupLayout;
}
