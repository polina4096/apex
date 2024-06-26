use rodio::{source::SeekError, Device, OutputStream, OutputStreamHandle, Sink, Source};

use super::OutputStreamExt as _;

pub struct AudioEngine {
  #[allow(unused)]
  stream: OutputStream,

  #[allow(unused)]
  stream_handle: OutputStreamHandle,

  device: Device,
  sink: Sink,
}

impl AudioEngine {
  pub fn new() -> Self {
    let (device, (stream, stream_handle)) = OutputStream::try_default_device().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    return Self { stream, stream_handle, device, sink };
  }

  pub fn set_source<S>(&mut self, source: S)
  where
    S: Source<Item = f32> + Send + 'static,
  {
    self.sink.clear();
    self.sink.append(source);
  }

  pub fn play(&self) {
    self.sink.play();
  }

  pub fn pause(&self) {
    self.sink.pause();
  }

  pub fn is_paused(&self) -> bool {
    return self.sink.is_paused();
  }

  pub fn try_seek(&mut self, pos: std::time::Duration) -> Result<(), SeekError> {
    return self.sink.try_seek(pos);
  }

  pub fn device(&self) -> &Device {
    return &self.device;
  }
}
