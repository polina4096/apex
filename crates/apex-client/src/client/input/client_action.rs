#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientAction {
  Back,

  Next,
  Prev,

  Select,
}
