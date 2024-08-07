use apex_framework::log::install_logger;

fn main() -> color_eyre::Result<()> {
  color_eyre::install()?;
  install_logger()?;

  let event_loop = apex_client::create_event_loop();
  apex_client::run(event_loop)?;

  return Ok(());
}
