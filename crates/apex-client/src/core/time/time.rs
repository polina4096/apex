use std::fmt::{Debug, Display};

use instant::{Duration, Instant};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Time(f64);

impl Time {
  pub const fn zero() -> Self {
    return Self(0.0);
  }

  pub fn from_seconds(value: impl Into<f64>) -> Self {
    return Time(value.into());
  }

  pub fn from_ms(value: impl Into<f64>) -> Self {
    return Time(value.into() / 1000.0);
  }

  pub fn to_seconds(&self) -> f64 {
    return self.0;
  }

  pub fn to_ms(&self) -> i64 {
    return (self.0 * 1000.0).round() as i64;
  }
}

impl From<Instant> for Time {
  fn from(value: Instant) -> Self {
    return Time(value.elapsed().as_secs_f64());
  }
}

impl std::ops::Add for Time {
  type Output = Time;

  fn add(self, rhs: Self) -> Self::Output {
    return Time(self.0 + rhs.0);
  }
}

impl std::ops::Sub for Time {
  type Output = Time;

  fn sub(self, rhs: Self) -> Self::Output {
    return Time(self.0 - rhs.0);
  }
}

impl std::ops::Mul for Time {
  type Output = Time;

  fn mul(self, rhs: Self) -> Self::Output {
    return Time(self.0 * rhs.0);
  }
}

impl std::ops::Div for Time {
  type Output = Time;

  fn div(self, rhs: Self) -> Self::Output {
    return Time(self.0 / rhs.0);
  }
}

impl std::ops::Rem for Time {
  type Output = Time;

  fn rem(self, rhs: Self) -> Self::Output {
    return Time(self.0 % rhs.0);
  }
}

impl std::ops::Mul<f64> for Time {
  type Output = Time;

  fn mul(self, rhs: f64) -> Self::Output {
    return Time(self.0 * rhs);
  }
}

impl std::ops::Div<f64> for Time {
  type Output = Time;

  fn div(self, rhs: f64) -> Self::Output {
    return Time(self.0 / rhs);
  }
}

impl From<Duration> for Time {
  fn from(value: Duration) -> Self {
    return Time(value.as_secs_f64());
  }
}

// Temporary
impl From<Time> for Duration {
  fn from(value: Time) -> Self {
    return Duration::from_secs_f64(value.0);
  }
}

impl Display for Time {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.0 < 0.0 {
      let duration: Duration = Time(self.0.abs()).into();
      write!(f, "-")?;
      return duration.fmt(f);
    }

    let duration: Duration = (*self).into();
    return duration.fmt(f);
  }
}
