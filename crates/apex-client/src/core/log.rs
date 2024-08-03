use std::{fs::File, io};

use color_eyre::eyre::Context;
use jiff::{fmt::strtime, Zoned};
use pretty_env_logger::env_logger::fmt::Target;

pub struct FileLogger {
  file: File,
}

impl FileLogger {
  pub fn new() -> color_eyre::Result<Self> {
    if !std::fs::exists("./logs").unwrap_or(false) {
      std::fs::create_dir_all("./logs").context("Unable to create log directory")?;
    }

    let now = strtime::format("%Y-%m-%dT%H:%M:%S", &Zoned::now()).context("Unable to format time")?;
    let file = File::create(format!("./logs/{}", now)).context("Unable to create a log file")?;

    return Ok(Self { file });
  }
}

impl io::Write for FileLogger {
  #[inline(always)]
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    return self.file.write(buf);
  }

  #[inline(always)]
  fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
    return self.file.write_vectored(bufs);
  }

  #[inline(always)]
  fn flush(&mut self) -> io::Result<()> {
    return self.file.flush();
  }
}

pub fn install_logger() -> color_eyre::Result<()> {
  let mut builder = pretty_env_logger::formatted_builder();

  builder.target(Target::Stdout);

  if atty::isnt(atty::Stream::Stdout) {
    let pipe = Target::Pipe(Box::new(FileLogger::new()?));
    builder.target(pipe);
  }

  if let Ok(s) = std::env::var("RUST_LOG") {
    builder.parse_filters(&s);
  }

  builder.try_init()?;
  return Ok(());
}
