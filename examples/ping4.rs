// Copyright 2021 Jeremy Wall
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    io::{stdin, stdout},
    io::{ErrorKind, Read},
    net::Ipv4Addr,
    process,
    time::{Duration, Instant},
};

use laska::packet::WithEchoRequest;
use laska::socket::IcmpSocket;
use laska::*;

pub fn main() {
    let address = std::env::args().nth(1).unwrap_or("127.0.0.1".to_owned());
    let parsed_addr = address.parse::<Ipv4Addr>().unwrap();
    let packet_handler = |pkt: Icmpv4Packet, send_time: Instant, addr: Ipv4Addr| -> Option<()> {
        let now = Instant::now();
        let elapsed = now - send_time;
        if addr == parsed_addr {
            // TODO
            if let Icmpv4Message::EchoReply {
                identifier: _,
                sequence,
                payload,
            } = pkt.message
            {
                println!(
                    "Ping {} seq={} time={}ms size={}",
                    addr,
                    sequence,
                    (elapsed.as_micros() as f64) / 1000.0,
                    payload.len()
                );
            } else {
                //eprintln!("Discarding non-reply {:?}", pkt);
                return None;
            }
            Some(())
        } else {
            eprintln!("Discarding packet from {}", addr);
            None
        }
    };

    let mut sock = match IcmpSocket4::new() {
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            eprintln!("please run with administrative priveleged mode");
            process::exit(0x01);
        }
        Err(e) => {
            eprintln!("{:?}", e);
            process::exit(0x01);
        }
        Ok(s) => s,
    };

    sock.bind("0.0.0.0".parse::<Ipv4Addr>().unwrap()).unwrap();
    sock.set_timeout(Some(Duration::from_secs(1)));
    let mut sequence = 0 as u16;

    // let stdin = stdin();
    // let stdout = stdout();
    let mut in_lock = stdin().lock();
    let _out_lock = stdout().lock();

    // loop {
    //     let mut buffer = [0; 10];
    //     let n = in_lock.read(&mut buffer[..]);

    //     println!("The bytes: {:?}", &buffer[..n]);
    //     Ok(())

    //     let b = b.unwrap();
    let mut buf: Vec<u8> = vec![];
    while in_lock.read(&mut buf).unwrap() > 0 {
        let packet = match Icmpv4Packet::with_echo_request(
            42,
            sequence,
            // buf.clone(),
            buf.clone(),
            // vec![
            //     0x20, 0x20, 0x75, 0x73, 0x74, 0x20, 0x61, 0x20, 0x66, 0x6c, 0x65, 0x73, 0x68, 0x20,
            //     0x77, 0x6f, 0x75, 0x6e, 0x64, 0x20, 0x20, 0x74, 0x69, 0x73, 0x20, 0x62, 0x75, 0x74,
            //     0x20, 0x61, 0x20, 0x73, 0x63, 0x72, 0x61, 0x74, 0x63, 0x68, 0x20, 0x20, 0x6b, 0x6e,
            //     0x69, 0x67, 0x68, 0x74, 0x73, 0x20, 0x6f, 0x66, 0x20, 0x6e, 0x69, 0x20, 0x20, 0x20,
            // ],
        ) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{:?}", e);
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        let send_time = Instant::now();
        sock.send_to(address.parse::<Ipv4Addr>().unwrap(), packet)
            .unwrap();

        loop {
            let (resp, sock_addr) = match sock.rcv_from() {
                Ok(tpl) => tpl,
                Err(e) => {
                    eprintln!("{:?}", e);
                    break;
                }
            };
            if packet_handler(resp, send_time, *sock_addr.as_socket_ipv4().unwrap().ip()).is_some()
            {
                std::thread::sleep(Duration::from_secs(1));
                break;
            }
        }
        sequence = sequence.wrapping_add(1);
    }
}
