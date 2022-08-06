use std::net::Ipv4Addr;
use std::ops::Range;
use std::str::FromStr;
use clap::Parser;
use tracing::{error, info};
use tcproxy_core::Result;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct AppArguments {
    #[clap(short, long, value_parser, default_value = "8080")]
    port: u16,

    #[clap(short, long, value_parser, default_value = "0.0.0.0")]
     ip: String,

    #[clap(short = 'D', long, value_parser = parse_port_range, default_value = "10000:25000")]
    port_range: String,
}

impl AppArguments {
    pub fn port(&self) -> u16 { self.port }
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

    pub fn parse_port_range(&self) -> Result<Range<u16>> {
        info!("{}", self.port_range);

        let groups: Vec<&str> = self.port_range.split(':').collect();
        let initial_port = groups[0].to_owned().parse::<u16>()?;
        let final_port = groups[1].to_owned().parse::<u16>()?;

        Ok(initial_port..final_port)
    }
}


fn parse_port_range(s: &str) -> Result<String> {
    let groups: Vec<&str> = s.split(':').collect();
    if 2 > groups.len() {
        return Err(format!("Invalid port range: {}", s).into());
    }

    let initial_port = groups[0].to_owned().parse::<u16>();
    let final_port = groups[1].to_owned().parse::<u16>();

    if initial_port.is_err() || final_port.is_err() {
        return Err(format!("Invalid por values.. {}", s).into());
    }

    Ok(String::from(s))
}
