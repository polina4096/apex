use std::fs::File;

use color_eyre::eyre::Context as _;
use jiff::{fmt::strtime, Zoned};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger};

fn main() -> color_eyre::Result<()> {
  color_eyre::install()?;

  if !std::fs::exists("./logs").unwrap_or(false) {
    std::fs::create_dir_all("./logs").context("Failed to create log directory")?;
  }

  let now = strtime::format("%Y_%m_%dT%H_%M_%S", &Zoned::now()).context("Failed to format time")?;
  let file = File::create(format!("./logs/{}.txt", now)).context("Failed to create a log file")?;

  let config = ConfigBuilder::new()
    .add_filter_allow_str("apex_client")
    .add_filter_allow_str("apex_framework")
    .build();

  CombinedLogger::init(vec![
    TermLogger::new(LevelFilter::Debug, config.clone(), TerminalMode::Mixed, ColorChoice::Auto),
    WriteLogger::new(LevelFilter::Debug, config, file),
  ])
  .expect("Failed to initialize logger");

  let event_loop = apex_client::create_event_loop();
  apex_client::run(event_loop)?;

  return Ok(());
}
