use std::{net::{Ipv4Addr, SocketAddrV4}, str::FromStr};

use tcproxy_core::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ClientArgs {
  #[clap(short, long, value_parser)]
  port: u16,

  #[clap(short, long, value_parser = parse_ip, default_value = "127.0.0.1")]
  ip: String,

  #[clap(long, value_parser, default_value = "false")]
  debug: bool
}

impl ClientArgs {
  pub fn is_debug(&self) -> bool {
    self.debug
  }

  pub fn parse_target_ip(&self) -> Ipv4Addr {
    Ipv4Addr::from_str(&self.ip).unwrap()
  }

  pub fn parse_socket_addr(&self) -> SocketAddrV4 {
    let ip = self.parse_target_ip();
    SocketAddrV4::new(ip, self.port)
  }
}

/// validates if given ip target is a valid ip.
fn parse_ip(s: &str) -> Result<String> {
  match Ipv4Addr::from_str(s) {
    Ok(_) => Ok(String::from(s)),
    Err(err) => Err("Invalid IP Address.".into()),
  }
}