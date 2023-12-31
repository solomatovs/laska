mod icmp;
use icmp::IcmpApp;

use std::io::{self, BufRead};
use clap::Parser;
use tracing_subscriber::EnvFilter;
use tracing::error;


/// stdin, stdout via icmp
#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
pub struct IcmpArgs {
    /// binding ip address
    #[arg(short)]
    bind_ip: String,
}

fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
  
  let args = IcmpArgs::parse();
  let app = IcmpApp::new(&args.bind_ip)?;

  let mut handle = io::stdin().lock();
  let mut ip = String::new();

  while handle.read_line(&mut ip).unwrap() > 0 {
    let mut ip = ip.trim().to_owned();
    
    if let Err(e) = app.ip_working(&mut handle, &ip) {
      error!("{}", e);
    }

    ip.clear();
  }

  Ok(())
}
