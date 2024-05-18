#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientAction {
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
