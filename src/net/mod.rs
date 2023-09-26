use std::str::FromStr;
use std::io::{self, Cursor, Error, BufRead};
use std::net::{IpAddr, Ipv6Addr};
use std::mem;

use tracing::{error, info};
use anyhow::{anyhow, Result, Context};
use byteorder::{LittleEndian, ReadBytesExt};
use libc::{
  socket, bind, close, sendto, recvfrom,
  AF_INET, AF_INET6, SOCK_RAW,
  ssize_t, c_int, in_addr, sockaddr, sockaddr_in, sockaddr_in6, socklen_t
};
use clap::Parser;

// Static values
static IPPROTO_ICMP: c_int = 1;

enum SockAddr {
  V4(sockaddr_in, socklen_t),
  V6(sockaddr_in6, socklen_t),
}

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


/// stdin, stdout via icmp
#[derive(Parser, Debug)]
#[command(author, about, long_about = None)]
pub struct IcmpArgs {
    /// Name of the network interface through which icmp packets will be received
    #[arg(short)]
    iface: String,
}

pub struct IcmpApp {
  sockv4: c_int,
}

impl Drop for IcmpApp {
  fn drop(&mut self) {
    let res = unsafe {close(self.sockv4)};
    if res == -1 {
      error!("{}", Error::last_os_error().to_string());
    }
  }
}

impl IcmpApp {
  pub fn new() -> Result<Self> {
    let ip = "127.0.0.1".to_string();
    let addr = Self::string_to_sockaddr(&ip).map_or_else(
      || Err(anyhow!(format!("Invalid IP-address: {}", ip))),
      |x| Ok(x)
    )?;

    let sockv4 = Self::new_icmpv4_socket().context("Could not create socket (v4)")?;
    Self::set_nonblock(sockv4)?;
    
    Self::bind_to_ip(sockv4, addr)?;
    
    Ok(IcmpApp {sockv4})
  }

  fn set_nonblock(fd: libc::c_int) -> Result<()> {
    unsafe {
      let flags = libc::fcntl(fd, libc::F_GETFL);
      let res = libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);

      if res == -1 {
        return Err(anyhow!(Error::last_os_error().to_string()));
      }

      Ok(())
    }
  }
  fn new_icmpv4_socket() -> Result<c_int> {
    let res = unsafe {
      socket(AF_INET, SOCK_RAW, IPPROTO_ICMP)
    };

    if res == -1 {
      return Err(anyhow!(Error::last_os_error().to_string()));
    }

    Ok(res)
  }

  fn bind_to_ip(socket: c_int, addr: SockAddr) -> Result<()> {
    let (addr, len) = match &addr {
      SockAddr::V4(input, len) => (
        (input as *const sockaddr_in) as *const sockaddr,
        *len,
      ),
      SockAddr::V6(input, len) => (
        (input as *const sockaddr_in6) as *const sockaddr,
        *len,
      ),
    };

    let res = unsafe {bind(socket, addr, len)};
    if res != 0 {
      return Err(anyhow!(Error::last_os_error()));
    }

    return Ok(());
  }

  fn init_in6_addr(addr: Ipv6Addr) -> libc::in6_addr {
    libc::in6_addr {
      s6_addr: addr.octets()
    }
  }
  
  fn string_to_sockaddr(ip: &str) -> Option<SockAddr> {
    let dest_ip = IpAddr::from_str(ip);
    if let Ok(IpAddr::V4(dest_ip)) = dest_ip {
      let mut ipcursor = Cursor::new(dest_ip.octets());
      let addr = sockaddr_in {
        sin_len: 128,
        sin_family: AF_INET as u8,
        sin_port: 0,
        sin_addr: in_addr { s_addr: ipcursor.read_u32::<LittleEndian>().unwrap() },
        sin_zero: [0; 8],
      };
      let len = mem::size_of::<sockaddr_in>() as socklen_t;
      return Some(SockAddr::V4(addr, len));
    } else if let Ok(IpAddr::V6(dest_ip)) = dest_ip {
      let addr = sockaddr_in6 {
        sin6_len: 0,
        sin6_flowinfo: 0,
        sin6_port: 0,
        sin6_scope_id: 0,
        sin6_family: AF_INET6 as u8,
        sin6_addr: Self::init_in6_addr(dest_ip),
      };
      let len = mem::size_of::<sockaddr_in6>() as socklen_t;
      return Some(SockAddr::V6(addr, len));
    }
    None
  }


  fn send_packet(
    socket: c_int,
    addr: &SockAddr,
    buf: &[u8],
  ) -> Result<ssize_t> {
    let res = match addr {
      SockAddr::V4(addr, len) => unsafe {
        sendto(
          socket,
          buf.as_ptr() as *const _,
          buf.len(),
          0,
          (addr as *const sockaddr_in) as *const _,
          *len,
        )
      }
      SockAddr::V6(_, _) => {
        todo!();
      }
    };

    if res == -1 {
      return Err(anyhow!(Error::last_os_error()))
    }

    Ok(res)
  }

  fn recv_packet(
    socket: c_int,
    buf: &mut [u8],
  ) -> Result<(ssize_t, sockaddr_in)> {
    let mut len: socklen_t = mem::size_of::<sockaddr_in>() as socklen_t;
    let mut addr = sockaddr_in {
       sin_len: 0,
       sin_family: 0,
       sin_port: 0,
       sin_addr: in_addr {
        s_addr: 0,
       },
       sin_zero: [0; 8],
    };

    let res = unsafe {
      let buf_ref = buf.as_ptr() as *mut _;
      let addr_ref = &mut addr as *mut sockaddr_in as *mut _;
      let addr_len_ref = &mut len as *mut _;

      recvfrom(
        socket,
        buf_ref,
        buf.len(),
        0,
        addr_ref,
        addr_len_ref,
      )
    };

    if res == -1 {
      return Err(anyhow!(Error::last_os_error()));
    }

    return Ok((res, addr));
  }

  pub fn start(self) -> Result<()> {
    // Read from STDIN
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    // Create new ICMP-header (IPv4 and IPv6)
    let identifier = 0;
    let sequence_number = 0;
    let mut icmp4header = ICMP4Header::echo_request(identifier, sequence_number).to_byte_array();

    // Initialize TokenBucketFilter for rate-limiting
    // let rate = 10;
    // let mut tbf = TokenBucketFilter::new(rate);
    let addr  = Self::string_to_sockaddr("127.0.0.1");
    let addr = addr.unwrap();

    // Send packets in a while loop from STDIN
    while handle.read_line(&mut buffer).unwrap() > 0 {
      // tbf.take();
      
      let result = Self::send_packet(self.sockv4, &addr, &icmp4header);
      if let Err(msg) = result {
        error!("Could not send packet: {}", msg);
      }

      let result = Self::recv_packet(self.sockv4, &mut icmp4header);
      match result {
        Err(e) => {
          error!("Could not send packet: {}", e);
        }
        Ok((_len, _sock)) => {
          // info!("{:?}: {:?}", sock, &icmp4header[..len]);
        }
      }

      buffer.clear();
    }
    Ok(())
  }
}
