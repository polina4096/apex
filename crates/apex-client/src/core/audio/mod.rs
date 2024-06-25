use rodio::{cpal::traits::HostTrait as _, OutputStream, OutputStreamHandle, StreamError};

pub mod audio_engine;
pub mod audio_mixer;

pub use audio_mixer::mixer;

pub trait OutputStreamExt {
  fn try_default_device() -> Result<(rodio::Device, (OutputStream, OutputStreamHandle)), StreamError>;
}

impl OutputStreamExt for OutputStream {
  fn try_default_device() -> Result<(rodio::Device, (OutputStream, OutputStreamHandle)), StreamError> {
    let device = rodio::cpal::default_host().default_output_device().ok_or(StreamError::NoDevice).unwrap();
    let (device, (stream, stream_handle)) =
      OutputStream::try_from_device(&device).map(|s_sh| (device, s_sh)).or_else(|original_error| {
        // default device didn't work, try other ones
        let mut devices = match rodio::cpal::default_host().output_devices() {
          Ok(device) => device,
          Err(_) => return Err(original_error),
        };

        return devices.find_map(|d| OutputStream::try_from_device(&d).ok().map(|x| (d, x))).ok_or(original_error);
      })?;

    return Ok((device, (stream, stream_handle)));
  }
}
