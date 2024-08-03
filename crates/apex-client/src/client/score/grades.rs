use std::fmt::{self, Display, Formatter};

use crate::core::graphics::color::Color;

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grade {
  SS = 0,
  S  = 1,
  A  = 2,
  B  = 3,
  C  = 4,
  D  = 5,
}

impl<T: Into<u8>> From<T> for Grade {
  fn from(value: T) -> Self {
    match value.into() {
      0 => Grade::SS,
      1 => Grade::S,
      2 => Grade::A,
      3 => Grade::B,
      4 => Grade::C,
      _ => Grade::D,
    }
  }
}

impl Display for Grade {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Grade::SS => write!(f, "SS"),
      Grade::S => write!(f, "S"),
      Grade::A => write!(f, "A"),
      Grade::B => write!(f, "B"),
      Grade::C => write!(f, "C"),
      Grade::D => write!(f, "D"),
    }
  }
}

impl Grade {
  pub fn color(&self) -> Color {
    #[rustfmt::skip] return match self {
      Grade::SS => Color::new(0.960, 0.825, 0.275, 1.0),
      Grade::S  => Color::new(0.950, 0.685, 0.235, 1.0),
      Grade::A  => Color::new(0.350, 0.860, 0.255, 1.0),
      Grade::B  => Color::new(0.115, 0.275, 0.625, 1.0),
      Grade::C  => Color::new(0.625, 0.235, 0.650, 1.0),
      Grade::D  => Color::new(0.825, 0.155, 0.235, 1.0),
    };
  }

  pub fn from_osu_stable(result_300: usize, result_150: usize, result_miss: usize) -> Self {
    if result_150 == 0 && result_miss == 0 {
      return Grade::SS;
    }

    let total = result_300 + result_150 + result_miss;

    // Grade "S"
    // 90% of hits are 300 and no misses
    if result_300 as f64 / total as f64 > 0.9 && result_miss == 0 {
      return Grade::S;
    }

    // Grade "A"
    // 80% of hits are 300 and no misses or 90% of hits are 300
    if result_300 as f64 / total as f64 > 0.8 && result_miss == 0 || result_300 as f64 / total as f64 > 0.9 {
      return Grade::A;
    }

    // Grade "B"
    // 70% of hits are 300 and no misses or 80% of hits are 300
    if result_300 as f64 / total as f64 > 0.7 && result_miss == 0 || result_300 as f64 / total as f64 > 0.8 {
      return Grade::B;
    }

    // Grade "C"
    // 60% of hits are 300
    if result_300 as f64 / total as f64 > 0.6 {
      return Grade::C;
    }

    return Grade::D;
  }
}
