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
//! An ICMP socket library that tries to be ergonomic to use.
//!
//! The standard ping examples for both Ipv6 and IPv4 are in the examples
//! directory.
pub mod packet;
pub mod pipe;
pub mod socket;

pub use packet::{Icmpv4Message, Icmpv4Packet, Icmpv6Message, Icmpv6Packet};
pub use pipe::{PipeError, PipeStreamReader, PipedLine};
pub use socket::{IcmpSocket, IcmpSocket4, IcmpSocket6};
