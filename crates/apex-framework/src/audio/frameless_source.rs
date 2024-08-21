use std::time::Duration;

use rodio::{Sample, Source};

pub struct FramelessSource<I>
where
  I: Source,
  I::Item: Sample,
{
  inner: I,
}

impl<I> FramelessSource<I>
where
  I: Source,
  I::Item: Sample,
{
  pub fn new(source: I) -> Self {
    return Self { inner: source };
  }
}

impl<I> From<I> for FramelessSource<I>
where
  I: Source,
  I::Item: Sample,
{
  fn from(value: I) -> Self {
    return Self::new(value);
  }
}

impl<I> Iterator for FramelessSource<I>
where
  I: Source,
  I::Item: Sample,
{
  type Item = I::Item;

  fn next(&mut self) -> Option<Self::Item> {
    return self.inner.next();
  }
}

impl<I> Source for FramelessSource<I>
where
  I: Source,
  I::Item: Sample,
{
  fn current_frame_len(&self) -> Option<usize> {
    return None;
  }

  fn channels(&self) -> u16 {
    return self.inner.channels();
  }

  fn sample_rate(&self) -> u32 {
    return self.inner.sample_rate();
  }

  fn total_duration(&self) -> Option<Duration> {
    return self.inner.total_duration();
  }
}
