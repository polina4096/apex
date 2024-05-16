use crate::core::time::time::Time;

#[derive(Clone, Default)]
pub struct HitObject {
  pub time  : Time,

  pub big   : bool,
  pub color : TaikoColor,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TaikoColor {
  #[default]
  DON,
  KAT,
}

impl TaikoColor {
  pub fn toggle(&mut self) {
    match self {
      TaikoColor::KAT => *self = TaikoColor::DON,
      TaikoColor::DON => *self = TaikoColor::KAT,
    }
  }
}
