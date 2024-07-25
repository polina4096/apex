use std::path::Path;

pub trait Persistant {
  fn load(path: impl AsRef<Path>) -> Self;
  fn save(&self, path: impl AsRef<Path>);
}
