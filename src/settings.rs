use anstyle;
use clap::builder::TypedValueParser as _;
use clap::Parser;
use log::LevelFilter;
use std::error::Error;
use std::net::ToSocketAddrs;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::num::ParseIntError;
use std::time::Duration;

use clap;

#[derive(Parser, Debug)] // requires `derive` feature
#[command(term_width = 0, styles=get_styles())] // Just to make testing across clap features easier
pub struct Args {
    /// Connect to IP
    #[arg(
        value_parser = parse_ip,
    )]
    pub to_ip: (IpAddr, Option<u16>),

    /// Binding IP addresses
    #[arg(
        short,
        long,
        default_value = "127.0.0.1",
        value_parser = parse_ip,
    )]
    pub bind_ip: (IpAddr, Option<u16>),

    #[arg(
        short,
        long,
        default_value = "5",
        value_parser = parse_duration,
    )]
    pub sleep: Duration,

    /// log level
    #[arg(
        short,
        long,
        default_value = "info",
        value_parser = clap::builder::PossibleValuesParser::new(["trace", "debug", "info", "warn", "error"]).map(|s| s.parse::<LevelFilter>().unwrap()),
    )]
    pub log_level: LevelFilter,
}

pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::White))),
        )
}

/// Parse a single key-value pair
fn _parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

/// Parse host/ip:port (or host/ip)
fn parse_ip(s: &str) -> Result<(IpAddr, Option<u16>), Box<dyn Error + Send + Sync + 'static>> {
    if let Ok(s) = s.to_socket_addrs() {
        for s in s {
            return Ok((s.ip(), Some(s.port())));
        }
    };

    if let Ok(s) = (s, 0).to_socket_addrs() {
        for s in s {
            return Ok((s.ip(), None));
        }
    };

    if let Ok(addr) = s.parse::<Ipv4Addr>() {
        return Ok((IpAddr::V4(addr), None));
    };

    if let Ok(addr) = s.parse::<Ipv6Addr>() {
        return Ok((IpAddr::V6(addr), None));
    };

    Err(format!("error parsing ip/host:port (or ip/host) from value: {s}").into())
}

fn parse_duration(arg: &str) -> Result<Duration, ParseIntError> {
    let millis = arg.parse()?;
    Ok(Duration::from_millis(millis))
}
