use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use clap::Parser;
use tcproxy_core::Result;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ClientArgs {
    #[clap(subcommand)]
    command_type: AppCommandType,
}

#[derive(clap::Subcommand, Debug)]
pub enum AppCommandType {
    Listen(ListenArgs),
    Config(ConfigArgs),
}

#[derive(Parser, Debug)]
pub struct ConfigArgs {
    #[clap(short, long, value_parser)]
    pub(crate) port: u16,
}

#[derive(Parser, Debug)]
pub struct ListenArgs {
    #[clap(short, long, value_parser)]
    port: u16,

    #[clap(short, long, value_parser = parse_ip, default_value = "127.0.0.1")]
    ip: Ipv4Addr,

    #[clap(short, long, value_parser, default_value = "false")]
    verbose: bool,

    #[clap(long, default_value = "5", value_parser = parse_ping_interval)]
    ping_interval: u8,
}

impl ClientArgs {
    pub fn get_type(&self) -> &AppCommandType {
        &self.command_type
    }
}

impl ListenArgs {
    pub fn is_debug(&self) -> bool {
        self.verbose
    }

    pub fn parse_socket_addr(&self) -> SocketAddrV4 {
        SocketAddrV4::new(self.ip, self.port)
    }

    pub fn ping_interval(&self) -> u8 {
        self.ping_interval
    }
}

impl Clone for ListenArgs {
    fn clone(&self) -> Self {
        Self {
            verbose: self.verbose,
            ip: self.ip,
            ping_interval: self.ping_interval,
            port: self.port,
        }
    }
}

fn parse_ping_interval(s: &str) -> Result<u8> {
    let parsed_value = match s.parse::<u8>() {
      Ok(value) => value,
      Err(err) => return Err(err.into()),
    };

    if 2 > parsed_value {
        return Err("minimum ping interval is 2s".into());
    }

    return Ok(parsed_value);
}

/// validates if given ip target is a valid ip.
fn parse_ip(s: &str) -> Result<Ipv4Addr> {
    match Ipv4Addr::from_str(s) {
        Ok(ip) => Ok(ip),
        Err(_) => Err("Invalid IP Address.".into()),
    }
}
