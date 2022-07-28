use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use clap::Parser;
use tracing::error;

use crate::Result;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct AppArguments {
    #[clap(short, long, value_parser, default_value = "8080")]
    port: i32,

    #[clap(short, long, value_parser, default_value = "0.0.0.0")]
     ip: String,

    #[clap(short = 'D', long, value_parser = parse_port_range, default_value = "10000:25000")]
    port_range: String,
}

impl AppArguments {
    pub fn port(&self) -> i32 { self.port }
    pub fn ip(&self) -> &str { &self.ip }
    pub fn port_range(&self) -> &str { &self.port_range }

    pub fn parse_ip(&self) -> Result<Ipv4Addr> {
        match Ipv4Addr::from_str(&self.ip) {
            Ok(ip) => Ok(ip),
            Err(err) => {
                error!("Failed when parsing IP Address: {}", err);
                Err(err.into())
            },
        }
    }
}


fn parse_port_range(s: &str) -> Result<String> {
    let groups: Vec<&str> = s.split(':').collect();
    if 2 > groups.len() {
        return Err(format!("Invalid port range: {}", s).into());
    }

    let initial_port = groups[0].to_owned().parse::<i32>();
    let final_port = groups[1].to_owned().parse::<i32>();

    if !initial_port.is_ok() || !final_port.is_ok() {
        return Err(format!("Invalid por values.. {}", s).into());
    }

    Ok(String::from("Test"))
}
