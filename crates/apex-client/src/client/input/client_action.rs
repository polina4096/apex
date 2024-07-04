#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientAction {
  Back,
  Settings,
  Recording,

  Next,
  Prev,

  Select,
  Retry,

  KatOne,
  DonOne,
  KatTwo,
  DonTwo,
}
