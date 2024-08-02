fn main() -> color_eyre::Result<()> {
  color_eyre::install()?;
  pretty_env_logger::init();

  let event_loop = apex_client::create_event_loop();
  apex_client::run(event_loop)?;

  return Ok(());
}
