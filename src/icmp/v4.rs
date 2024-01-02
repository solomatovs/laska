use std::io::Error;

use anyhow::{anyhow, Result};
use tracing::error;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{BufRead, Cursor, StdinLock};
use std::mem;
use std::net::Ipv4Addr;

use libc::{
    bind, c_int, close, in_addr, recvfrom, sendto, sockaddr, sockaddr_in, socket, socklen_t,
    ssize_t, AF_INET, IPPROTO_ICMP, SOCK_RAW,
};

pub struct IcmpV4App {
    socket: c_int,
    _addr: sockaddr_in,
    _len: socklen_t,
}

impl Drop for IcmpV4App {
    fn drop(&mut self) {
        let res = unsafe { close(self.socket) };

        if res == -1 {
            error!("{}", Error::last_os_error().to_string());
        }
    }
}

impl IcmpV4App {
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

    pub fn new(sockaddr: sockaddr_in, len: socklen_t) -> Result<IcmpV4App> {
        let socket = unsafe { socket(AF_INET, SOCK_RAW, IPPROTO_ICMP) };

        if socket == -1 {
            return Err(anyhow!(Error::last_os_error().to_string()));
        }

        Self::set_nonblock(socket)?;

        let res = unsafe {
            let addr_ref = &sockaddr as *const sockaddr_in;
            let addr_ref = addr_ref as *const sockaddr;
            bind(socket, addr_ref, len)
        };

        if res == -1 {
            return Err(anyhow!(Error::last_os_error().to_string()));
        }

        Ok(IcmpV4App {
            socket,
            _addr: sockaddr,
            _len: len,
        })
    }

    pub fn ip_to_sockaddr(dest_ip: Ipv4Addr) -> Result<IcmpV4App> {
        let mut ipcursor = Cursor::new(dest_ip.octets());
        let addr = sockaddr_in {
            sin_len: 0,
            sin_family: AF_INET as u8,
            sin_port: 0,
            sin_addr: in_addr {
                s_addr: ipcursor.read_u32::<LittleEndian>().unwrap(),
            },
            sin_zero: [0; 8],
        };
        let len = mem::size_of::<sockaddr_in>() as socklen_t;
        Self::new(addr, len)
    }

    fn _bind_to_ip(socket: c_int, addr: &sockaddr_in, len: socklen_t) -> Result<()> {
        let addr_ref = (addr as *const sockaddr_in) as *const sockaddr;
        let res = unsafe { bind(socket, addr_ref, len) };

        if res != 0 {
            return Err(anyhow!(Error::last_os_error()));
        }

        return Ok(());
    }

    pub fn send_packet(
        self,
        addr: &sockaddr_in,
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
                (addr as *const sockaddr_in) as *const _,
                addr_len,
            )
        };

        if res == -1 {
            return Err(anyhow!(Error::last_os_error()));
        }

        Ok(res)
    }

    pub fn recv_packet(self, buf: &mut [u8], flags: c_int) -> Result<(ssize_t, sockaddr_in)> {
        let mut len: socklen_t = self._len;

        let mut addr = sockaddr_in {
            sin_len: 0,
            sin_family: AF_INET as u8,
            sin_port: 0,
            sin_addr: in_addr { s_addr: 0 },
            sin_zero: [0; 8],
        };

        let res = unsafe {
            let buf_ref = buf.as_ptr() as *mut _;
            let addr_ref = &mut addr as *mut sockaddr_in as *mut _;
            let addr_len_ref = &mut len as *mut _;

            recvfrom(
                self.socket,
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
