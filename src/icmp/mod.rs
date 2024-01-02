mod v4;
mod v6;

use v4::IcmpV4App;
use v6::IcmpV6App;

use std::io::{BufRead, Cursor, StdinLock};
use std::mem;
use std::net::{IpAddr, Ipv6Addr};
use std::str::FromStr;

use anyhow::{Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use libc::{in_addr, sockaddr_in, sockaddr_in6, socklen_t, AF_INET, AF_INET6};

#[derive(Debug)]
pub struct ICMP4Header {
    pub icmp_type: u8,
    pub code: u8,
    pub checksum: u16,
    pub header: u32,
}

impl ICMP4Header {
    pub fn echo_request(identifier: u16, sequence_number: u16) -> ICMP4Header {
        let header = ((identifier as u32) << 16) | (sequence_number as u32);
        let mut icmp4_header = ICMP4Header {
            icmp_type: 8,
            code: 0,
            checksum: 0,
            header,
        };
        let checksum = ICMP4Header::calc_checksum(&icmp4_header.to_byte_array());
        icmp4_header.checksum = checksum;
        icmp4_header
    }

    pub fn to_byte_array(&self) -> [u8; 8] {
        let mut buffer = [0; 8];
        buffer[0] = self.icmp_type;
        buffer[1] = self.code;
        buffer[2] = (self.checksum >> 8 & 0xFF) as u8;
        buffer[3] = (self.checksum & 0xFF) as u8;
        buffer[4] = (self.header >> 24 & 0xFF) as u8;
        buffer[5] = (self.header >> 16 & 0xFF) as u8;
        buffer[6] = (self.header >> 8 & 0xFF) as u8;
        buffer[7] = (self.header & 0xFF) as u8;
        buffer
    }

    fn calc_checksum(buffer: &[u8]) -> u16 {
        let mut size = buffer.len();
        let mut checksum: u32 = 0;
        while size > 0 {
            let word = (buffer[buffer.len() - size] as u16) << 8
                | (buffer[buffer.len() - size + 1]) as u16;
            checksum += word as u32;
            size -= 2;
        }
        let remainder = checksum >> 16;
        checksum &= 0xFFFF;
        checksum += remainder;
        checksum ^= 0xFFFF;
        checksum as u16
    }
}


pub enum IcmpApp {
    V4(IcmpV4App),
    V6(IcmpV6App),
}

impl IcmpApp {
    pub fn new(ip: &str) -> Result<IcmpApp> {
        let dest_ip = IpAddr::from_str(ip)?;

        match dest_ip {
            IpAddr::V4(dest_ip) => {
                let res =
                    IcmpV4App::ip_to_sockaddr(dest_ip).context("Could not create socket (v4)")?;
                Ok(IcmpApp::V4(res))
            }
            IpAddr::V6(dest_ip) => {
                let res =
                    IcmpV6App::ip_to_sockaddr(dest_ip).context("Could not create socket (v6)")?;
                Ok(IcmpApp::V6(res))
            }
        }
    }

    pub fn send_packet(&self, _buf: &String) -> Result<()> {
        match &self {
            Self::V4(_s) => {

            }
            Self::V6(_s) => {

            }
        }
        // todo!();
        // match &self {
        //     Self::V4(_a) => _a.send_packet(addr, addr_len, buf, flags),
        //     Self::V6(_a) => {}
        // }

        Ok(())
    }

    pub fn ip_working(&self, handle: &mut StdinLock, ip: &str) -> Result<()> {
        let _addr = Self::new(ip)?;

        // Create new ICMP-header (IPv4 and IPv6)
        let identifier = 0;
        let sequence_number = 0;
        let mut _header = ICMP4Header::echo_request(identifier, sequence_number).to_byte_array();
        let mut buf = String::new();

        while handle.read_line(&mut buf).unwrap() > 0 {
            self.send_packet(&buf)?;
            // match &self {
            //     Self::V4(_a) => _a.send_packet(addr, addr_len, buf, flags),
            //     Self::V6(_a) => {}
            // }
            buf.clear();
        }

        Ok(())
    }
}
