fn main() -> color_eyre::Result<()> {
  pretty_env_logger::init();

  let (event_loop, window) = apex_client::setup();
  apex_client::run(event_loop, window)?;

  return Ok(());
}
