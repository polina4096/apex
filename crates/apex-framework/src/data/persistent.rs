use std::path::Path;

pub trait Persistent {
  fn load(path: impl AsRef<Path>) -> Self;
  fn save(&self, path: impl AsRef<Path>);
}
