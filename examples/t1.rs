use anyhow::{anyhow, Result};

use libc::fcntl;
use log::{debug, error, info, trace};
use std::{
    io::{self, stdin, stdout, Error, ErrorKind, Read, Write},
    net::Ipv4Addr,
    os::fd::AsRawFd,
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use laska::packet::WithEchoRequest;
use laska::socket::IcmpSocket;
use laska::*;

fn main() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .try_init();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    if let Err(e) = ctrlc::set_handler(move || {
        info!("exit signal capture");
        r.store(false, Ordering::SeqCst);
    }) {
        error!("Error setting Ctrl-C handler");
        error!("{:?}", e);
        exit(1);
    }

    let check_mills = Duration::from_millis(5);
    let address = std::env::args().nth(1).unwrap_or("127.0.0.1".to_owned());

    let mut sock = match IcmpSocket4::new() {
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            error!("please run with administrative priveleged mode");
            exit(1);
        }
        Err(e) => {
            error!("error create socket");
            error!("{:?}", e);
            exit(1);
        }
        Ok(s) => s,
    };

    sock.bind("0.0.0.0".parse::<Ipv4Addr>().unwrap()).unwrap();
    sock.set_timeout(Some(Duration::from_secs(1)));
    let mut sequence = 0;

    let mut stdin = stdin();
    if let Err(e) = set_nonblock(&mut stdin) {
        error!("failed to switch to non-blocking mode of reading the standard input (stdin)");
        error!("{:?}", e);
        exit(1);
    }

    let mut data = [0u8; 512];

    while running.load(Ordering::SeqCst) {
        let read_len = match stdin.read(&mut data) {
            Ok(n) => n,
            Err(e) if e.kind() == ErrorKind::WouldBlock => 0,
            Err(e) => {
                error!("error reading stdin");
                error!("{:?}", e);
                0
            }
        };

        if read_len > 0 {
            match Icmpv4Packet::with_echo_request(42, sequence, data[..read_len].to_vec()) {
                Ok(p) => match sock.send_to(address.parse::<Ipv4Addr>().unwrap(), p) {
                    Ok(_) => {
                        sequence = sequence.wrapping_add(1);
                    }
                    Err(e) => {
                        error!("error send icmp packet");
                        error!("{:?}", e);
                    }
                },
                Err(e) => {
                    error!("error building icmp packet");
                    error!("{:?}", e);
                }
            };
        }

        match sock.rcv_from() {
            Ok((resp, sock_addr)) => {
                trace!("{:?}", sock_addr);
                trace!("{:?}", resp);

                match resp.message {
                    Icmpv4Message::EchoReply {
                        identifier: _,
                        sequence: _,
                        payload,
                    } => {
                        if let Err(_) = stdout().write_all(&payload) {};
                    }
                    _ => {
                        debug!("recv message: {:?}", resp.message);
                    }
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {}
            Err(e) => {
                error!("error recv socket");
                error!("{:?}", e);
            }
        };

        std::thread::sleep(check_mills);
    }

    info!("💪(◡̀_◡́҂)")
}

fn set_nonblock(stream: &mut io::Stdin) -> Result<()> {
    let fd = stream.as_raw_fd();
    unsafe {
        let flags = fcntl(fd, libc::F_GETFL);
        let res = fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);

        if res == -1 {
            return Err(anyhow!(Error::last_os_error().to_string()));
        }

        Ok(())
    }
}
