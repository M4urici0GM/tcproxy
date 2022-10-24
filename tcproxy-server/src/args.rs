use std::{net::{IpAddr, Ipv4Addr}, ops::Range};

use clap::Parser;
use tcproxy_core::Result;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct AppArguments {
    #[clap(short, long, value_parser)]
    port: Option<u16>,

    #[clap(short, long, value_parser)]
    ip: Option<IpAddr>,

    #[clap(short = 'D', long, value_parser = parse_port_range)]
    port_range: Option<Range<u16>>,

    #[clap(long = "max-connections-per-proxy")]
    max_connections_per_proxy: Option<u16>
}

impl AppArguments {
    pub fn new(
        port: Option<u16>,
        ip: Option<IpAddr>,
        port_range: Option<Range<u16>>,
        max_connections_per_proxy: Option<u16>
    ) -> Self {
        Self {
            port,
            ip,
            port_range,
            max_connections_per_proxy,
        }
    }

    pub fn get_port(&self) -> Option<u16> {
        self.port
    }

    pub fn get_ip(&self) -> Option<IpAddr> {
        self.ip
    }

    pub fn get_port_range(&self) -> Option<Range<u16>> {
        self.port_range.clone()
    }

    pub fn get_max_connections_per_proxy(&self) -> Option<u16> {
        self.max_connections_per_proxy
    }
}

impl Default for AppArguments {
    fn default() -> Self {
        Self {
            port: None,
            ip: None,
            port_range: None,
            max_connections_per_proxy: None,
        }
    }
}

fn parse_port_range(s: &str) -> Result<Range<u16>> {
    let groups: Vec<&str> = s.split(':').collect();
    if 2 > groups.len() {
        return Err(format!("Invalid port range: {}", s).into());
    }

    let initial_port = groups[0].to_owned().parse::<u16>();
    let final_port = groups[1].to_owned().parse::<u16>();

    if initial_port.is_err() || final_port.is_err() {
        return Err(format!("Invalid por values.. {}", s).into());
    }

    Ok(initial_port?..final_port?)
}
