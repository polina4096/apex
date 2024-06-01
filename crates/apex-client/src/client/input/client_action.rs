#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientAction {
  Settings,
  Back,

  Next,
  Prev,

  Select,
  Retry,

  KatOne,
  DonOne,
  KatTwo,
  DonTwo,
}
