use super::graphics::Graphics;

pub trait Drawable {
  fn recreate(&mut self, graphics: &Graphics);
}
