use std::io::Error;
use std::mem;

use tracing::error;
use anyhow::{anyhow, Result};
use libc::{
  socket, bind, close, sendto, recvfrom,
  AF_INET6, SOCK_RAW,
  ssize_t, c_int, in6_addr, sockaddr, sockaddr_in6, socklen_t
};

static IPPROTO_ICMP: c_int = 1;

pub struct IcmpV6App {
  socket: c_int,
  _addr: sockaddr_in6,
  _len: socklen_t,
}

impl Drop for IcmpV6App {
  fn drop(&mut self) {
    let res = unsafe {close(self.socket)};
    
    if res == -1 {
      error!("{}", Error::last_os_error().to_string());
    }
  }
}

impl IcmpV6App {
  fn set_nonblock(socket: c_int) -> Result<()> {
    unsafe {
      let flags = libc::fcntl(socket, libc::F_GETFL);
      let res = libc::fcntl(socket, libc::F_SETFL, flags | libc::O_NONBLOCK);
  
      if res == -1 {
        return Err(anyhow!(Error::last_os_error().to_string()));
      }
  
      Ok(())
    }
  }

  pub fn new(addr: sockaddr_in6, len: socklen_t) -> Result<IcmpV6App> {
    let socket = unsafe {
      socket(AF_INET6, SOCK_RAW, IPPROTO_ICMP)
    };

    if socket == -1 {
      return Err(anyhow!(Error::last_os_error().to_string()));
    }

    Self::set_nonblock(socket)?;
    
    let res = unsafe {
      let addr_ref = &addr as *const sockaddr_in6;
      let addr_ref = addr_ref as *const sockaddr;
      bind(socket, addr_ref, len)
    };

    if res == -1 {
      return Err(anyhow!(Error::last_os_error().to_string()));
    }

    Ok(IcmpV6App {
      socket,
      _addr: addr,
      _len: len,
    })
  }

  fn _bind_to_ip(socket: c_int, addr: &sockaddr_in6, len: socklen_t) -> Result<()> {
    let addr_ref = (addr as *const sockaddr_in6) as *const sockaddr;
    let res = unsafe {
      bind(socket, addr_ref, len)
    };
    
    if res != 0 {
      return Err(anyhow!(Error::last_os_error()));
    }

    return Ok(());
  }

  fn _send_packet(self,
    addr: &sockaddr_in6,
    addr_len: socklen_t,
    buf: &[u8],
    flags: c_int,
  ) -> Result<ssize_t> {
    let res = unsafe {
      sendto(
        self.socket,
        buf.as_ptr() as *const _,
        buf.len(),
        flags,
        (addr as *const sockaddr_in6) as *const _,
        addr_len,
      )
    };

    if res == -1 {
      return Err(anyhow!(Error::last_os_error()))
    }

    Ok(res)
  }
  
  fn _recv_packet(self, buf: &mut [u8], flags: c_int) -> Result<(ssize_t, sockaddr_in6)> {
    let mut len: socklen_t = mem::size_of::<sockaddr_in6>() as socklen_t;
    let mut addr = sockaddr_in6 {
      sin6_family: AF_INET6 as u16,
      sin6_port: 0,
      sin6_flowinfo: 0,
      sin6_addr: in6_addr {
        s6_addr: [0; 16],
      },
      sin6_scope_id: 0,
    };

    let res = unsafe {
      let buf_ref = buf.as_ptr() as *mut _;
      let addr_ref = &mut addr as *mut sockaddr_in6 as *mut _;
      let addr_len_ref = &mut len as *mut _;

      recvfrom(self.socket,
        buf_ref,
        buf.len(),
        flags,
        addr_ref,
        addr_len_ref,
      )
    };

    if res == -1 {
      return Err(anyhow!(Error::last_os_error()));
    }

    return Ok((res, addr));
  }
}
