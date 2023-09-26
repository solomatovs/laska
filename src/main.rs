mod net;
use tracing_subscriber::EnvFilter;


fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
  
  let app = net::IcmpApp::new()?;

  return app.start();
}
