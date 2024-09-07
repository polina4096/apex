use rodio::{cpal::traits::HostTrait, DeviceTrait};
use serde::{Deserialize, Serialize};

pub mod game_audio;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioOutput(String);

impl AudioOutput {
  pub fn new(value: impl Into<String>) -> Self {
    return Self(value.into());
  }

  pub fn as_str_pretty(&self) -> &str {
    return if self.0.is_empty() { "Default" } else { &self.0 };
  }

  pub fn as_str(&self) -> &str {
    return self.0.as_str();
  }

  pub fn device(&self) -> Option<rodio::Device> {
    return rodio::cpal::default_host()
      .output_devices()
      .ok()?
      .find(|x| x.name().map(|x| x == self.as_str()).unwrap_or(false))
      .or_else(|| rodio::cpal::default_host().default_output_device());
  }
}

impl std::fmt::Display for AudioOutput {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return write!(f, "{}", self.as_str_pretty());
  }
}
