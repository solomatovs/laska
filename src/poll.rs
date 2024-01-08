extern crate libc;

use std::result::Result;
use std::os::unix::io::RawFd;


/// Event types that can be polled for.  These bits may be set in `events'
/// to indicate the interesting event types; they will appear in `revents'
/// to indicate the status of the file descriptor.

/// There is data to read.
pub const POLLIN:   libc::c_short = 0x001;

/// There is urgent data to read.
pub const POLLPRI:  libc::c_short = 0x002;

/// Writing now will not block.
pub const POLLOUT:  libc::c_short = 0x004;


/// Event types always implicitly polled for.  These bits need not be set in
/// `events', but they will appear in `revents' to indicate the status of
/// the file descriptor.  */

/// Error condition.
pub const POLLERR:  libc::c_short = 0x008;

/// Hung up.
pub const POLLHUP:  libc::c_short = 0x010;

/// Invalid polling request.
pub const POLLNVAL: libc::c_short = 0x020;


/// `nfds_t` as defined in poll.h
#[allow(non_camel_case_types)]
type nfds_t = libc::c_ulong;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct PollFd {
    pub fd: RawFd,		 // file descriptor to poll
    pub events: libc::c_short, // types of events poller cares about
    pub revents: libc::c_short // types of events that actually occurred
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PollResult {
    Some(i32),
    Timeout
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PollError {
    /// The array given as argument was not contained in the calling program's address space.
    EFAULT,
    /// A signal occurred before any requested event; see signal(7).
    EINTR,
    /// The nfds value exceeds the RLIMIT_NOFILE value.
    EINVAL,
    /// There was no space to allocate file descriptor tables.
    ENOMEM
}

extern "C" {
    fn poll(__fds: *const PollFd, __nfds: nfds_t, __timeout: libc::c_int) -> libc::c_int;
}


#[allow(unused_mut)]
#[inline]
pub fn poll_wrapper(mut fds: &[PollFd], timeout: Option<i32>) -> Result<PollResult, PollError> {
    let __timeout = match timeout {
        Some(t) => t,
        None => -1
    } as libc::c_int;

    let retcode = unsafe {
        poll(fds.as_ptr(), fds.len() as nfds_t, __timeout)
    };

    if retcode == -1 {
        return Err(Error::last_os_error());
        // let errno = unsafe { *libc::__errno_location() };
        // match errno {
        //     libc::EFAULT => Err(PollError::EFAULT),
        //     libc::EINTR => Err(PollError::EINTR),
        //     libc::EINVAL => Err(PollError::EINVAL),
        //     libc::ENOMEM => Err(PollError::ENOMEM),
        //     x => panic!("unexpecter errno {}", x)
        // }
    } else if retcode == 0 {
        Ok(PollResult::Timeout)
    } else if retcode > 0 {
        Ok(PollResult::Some(retcode))
    } else {
        unreachable!()
    }
}
