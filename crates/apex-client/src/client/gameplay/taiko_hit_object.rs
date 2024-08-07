use apex_framework::time::time::Time;

#[rustfmt::skip]
#[derive(Clone, Default)]
pub struct TaikoHitObject {
  pub time  : Time,
  pub color : TaikoColor,
  pub big   : bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TaikoColor {
  #[default]
  Don,
  Kat,
}

impl TaikoColor {
  pub fn toggle(&mut self) {
    match self {
      TaikoColor::Kat => *self = TaikoColor::Don,
      TaikoColor::Don => *self = TaikoColor::Kat,
    }
  }

  pub fn is_don(&self) -> bool {
    return *self == TaikoColor::Don;
  }

  pub fn is_kat(&self) -> bool {
    return *self == TaikoColor::Kat;
  }
}
