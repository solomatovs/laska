use anyhow::anyhow;
use clap::Parser;
use log::{trace, info, debug, error, warn};
use std::{
    io::{self, stdin, stdout, Error, ErrorKind, Read, Write},
    mem,
    net::{Ipv4Addr, IpAddr},
    os::fd::AsRawFd,
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use anyhow::{Result, Context};

use laska::{
    settings::*,
    IcmpSocket4,
    *, os_notifier::OsNotifier,
};


fn main() -> Result<()> {
    let args = Args::parse();

    env_logger::builder()
        .parse_default_env()
        .filter_level(args.log_level)
        .init();

    trace!("{args:?}");

    let os_notifier = OsNotifier::new()?;

    let icmp_app = IcmpAppV4::new(&args, os_notifier.clone());

    std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}

pub struct  IcmpV4Args {
    sleep: Duration,
    to_ip: IpAddr,
}

impl From<&Args> for IcmpV4Args {
    fn from(value: &Args) -> Self {
        Self {
            to_ip: value.to_ip.0,
            sleep: value.sleep,
        }
    }
}

pub struct IcmpAppV4 {
   args: IcmpV4Args,
   socket: IcmpSocket4,
   shutdown: OsNotifier,
}

impl IcmpAppV4 {
    pub fn new(args: impl Into<IcmpV4Args>, shutdown: OsNotifier) -> Result<Self> {
        let base_error = "IcmpApp4 error create";

        let mut socket = match IcmpSocket4::new() {
            Err(e) if e.kind() == ErrorKind::PermissionDenied => {
                return Err(e)
                .context(base_error)
                .context("please run with administrative priveleged mode");
            }
            Err(e) => {
                return Err(e)
                .context(base_error)
                .context("failed to create a new ICMP socket");
            }
            Ok(s) => s,
        };
        
        let bind_addr = "0.0.0.0";
        socket.bind(bind_addr.parse::<Ipv4Addr>().unwrap())
            .context(base_error)
            .context(format!("ip address {bind_addr} binding error"))?;

        socket.set_timeout(Some(Duration::from_secs(1)));

        Ok(IcmpAppV4 {
            args: args.into(),
            shutdown,
            socket,
        })
    }

    pub fn work(&mut self) {
        let mut data = [0u8; 512];
        
        while !self.shutdown.is_kill_shutdown() {
            match self.socket.rcv_from() {
                Ok((resp, sock_addr)) => {
                    debug!("recv ICMP packet from {:?}", sock_addr);
                    trace!("{:?}", resp);
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {}
                Err(e) => {
                    error!("error recv socket");
                    error!("{:?}", e);
                }
            };
    
            std::thread::sleep(self.args.sleep);
        }
    }
}