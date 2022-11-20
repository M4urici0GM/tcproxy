use std::{str::FromStr, error::Error, fmt::Display, num::ParseIntError, net::{IpAddr, Ipv4Addr}};

use regex::Regex;


const IP_REGEX: &str = r"^\b(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b$";

#[derive(Debug)]
pub enum ServerAddrError {
    InvalidString,
    InvalidPort,
    Other(tcproxy_core::Error)
}

#[derive(Debug, Clone, Copy)]
pub enum ServerAddrType {
    IpAddr,
    DomainName
}

#[derive(Debug, Clone)]
pub struct ServerAddr {
    host: String,
    port: u16,
    addr_type: ServerAddrType,
}

impl ServerAddr {
    pub fn new(host: &str, port: &u16, raw_str: &str) -> Self {
        let addr_type = match Self::is_ip(host) {
            true => ServerAddrType::IpAddr,
            _ => ServerAddrType::DomainName,
        };

        Self {
            host: host.to_owned(),
            port: port.to_owned(),
            addr_type
        }
    }

    pub fn is_ip(host: &str) -> bool {
        Regex::new(IP_REGEX)
            .unwrap()
            .is_match(host)
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> &u16 {
        &self.port
    }

    pub fn addr_type(&self) -> ServerAddrType {
        self.addr_type
    }
}

impl FromStr for ServerAddr {
    type Err = ServerAddrError;

    fn from_str(given_str: &str) -> Result<Self, Self::Err> {
        let groups: Vec<&str> = given_str.split(":").collect();
        if 2 != groups.len() {
            return Err(ServerAddrError::InvalidString);
        }

        let host = groups[0];
        let port = match groups[1].parse::<u16>() {
            Ok(port) => port,
            Err(_) => return Err(ServerAddrError::InvalidPort),
        };

        let obj = ServerAddr::new(host, &port, given_str);

        Ok(obj)
    }
}

impl Error for ServerAddrError {
    
}

impl Display for ServerAddrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ServerAddrError::InvalidPort => format!("invalid port."),
            ServerAddrError::InvalidString => format!("invalid host string"),
            ServerAddrError::Other(err) => format!("unexpected error: {}", err),
        };

        write!(f, "{}", msg)
    }
}

impl From<ParseIntError> for ServerAddrError {
    fn from(err: ParseIntError) -> Self {
        ServerAddrError::Other(err.into())
    }
}