mod v4;
mod v6;

use v4::IcmpV4App;
use v6::IcmpV6App;

use std::str::FromStr;
use std::io::{Cursor, BufRead, StdinLock};
use std::net::{IpAddr, Ipv6Addr};
use std::mem;


use anyhow::{Result, Context};
use byteorder::{LittleEndian, ReadBytesExt};
use libc::{
  AF_INET, AF_INET6,
  in_addr, sockaddr_in, sockaddr_in6, socklen_t
};


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
      let word = (buffer[buffer.len() - size] as u16) << 8 | (buffer[buffer.len() - size + 1]) as u16;
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



enum SockAddr {
  V4(sockaddr_in, socklen_t),
  V6(sockaddr_in6, socklen_t),
}

pub enum IcmpApp {
  V4(IcmpV4App),
  V6(IcmpV6App),
}

impl IcmpApp {
  fn init_in6_addr(addr: Ipv6Addr) -> libc::in6_addr {
    libc::in6_addr {
      s6_addr: addr.octets()
    }
  }

  fn parse_ip(ip: &str) -> Result<SockAddr> {
    let dest_ip = IpAddr::from_str(ip)?;
    
    match dest_ip {
      IpAddr::V4(dest_ip) => {
        let mut ipcursor = Cursor::new(dest_ip.octets());
        let addr = sockaddr_in {
          sin_len: 128,
          sin_family: AF_INET as u8,
          sin_port: 0,
          sin_addr: in_addr { s_addr: ipcursor.read_u32::<LittleEndian>().unwrap() },
          sin_zero: [0; 8],
        };
        let len = mem::size_of::<sockaddr_in>() as socklen_t;
        return Ok(SockAddr::V4(addr, len));
      }
      IpAddr::V6(dest_ip) => {
        let addr = sockaddr_in6 {
          sin6_len: 0,
          sin6_flowinfo: 0,
          sin6_port: 0,
          sin6_scope_id: 0,
          sin6_family: AF_INET6 as u8,
          sin6_addr: Self::init_in6_addr(dest_ip),
        };
        let len = mem::size_of::<sockaddr_in6>() as socklen_t;
        return Ok(SockAddr::V6(addr, len));
      }
    }
  }

  pub fn new(ip: &str) -> Result<IcmpApp> {
    match Self::parse_ip(ip)? {
      SockAddr::V4(addr, len) => {
        let res = IcmpV4App::new(addr, len).context("Could not create socket (v4)")?;
        return Ok(IcmpApp::V4(res))
      }
      SockAddr::V6(addr, len) => {
        let res = IcmpV6App::new(addr, len).context("Could not create socket (v6)")?;
        return Ok(IcmpApp::V6(res))
      }
    }
  }

  pub fn ip_working(&self, handle: &mut StdinLock, ip: &str) -> Result<()> {
    let _addr  = Self::parse_ip(ip)?;

    // Create new ICMP-header (IPv4 and IPv6)
    let identifier = 0;
    let sequence_number = 0;
    let mut _icmp4header = ICMP4Header::echo_request(identifier, sequence_number).to_byte_array();
    let mut buf = String::new();

    while handle.read_line(&mut buf).unwrap() > 0 {

      buf.clear();
    }
    Ok(())
  }
}
