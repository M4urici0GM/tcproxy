use std::{str::FromStr, fmt::Display, num::ParseIntError};
use std::error::Error;

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
    pub fn new(host: &str, port: &u16) -> Result<Self, ServerAddrError> {
        Ok(Self {
            host: host.to_owned(),
            port: port.to_owned(),
            addr_type: Self::parse_type(host)?
        })
    }

    fn parse_type(host: &str) -> Result<ServerAddrType, ServerAddrError> {
        match Self::is_ip(host)? {
            true => Ok(ServerAddrType::IpAddr),
            _ => Ok(ServerAddrType::DomainName),
        }
    }

    fn is_ip(host: &str) -> Result<bool, ServerAddrError> {
        Ok(Regex::new(IP_REGEX)?.is_match(host))
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
        let groups: Vec<&str> = given_str.split(':').collect();
        if 2 != groups.len() {
            return Err(ServerAddrError::InvalidString);
        }

        let host = groups[0];
        let port = match groups[1].parse::<u16>() {
            Ok(port) => port,
            Err(_) => return Err(ServerAddrError::InvalidPort),
        };

        let obj = ServerAddr::new(host, &port)?;

        Ok(obj)
    }
}

impl Error for ServerAddrError {
    
}

impl From<ParseIntError> for ServerAddrError {
    fn from(err: ParseIntError) -> Self {
        Self::Other(err.into())
    }
}

impl From<regex::Error> for ServerAddrError {
    fn from(err: regex::Error) -> Self {
        Self::Other(err.into())
    }
}

impl Display for ServerAddrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ServerAddrError::InvalidPort => "invalid port.".to_string(),
            ServerAddrError::InvalidString => "invalid host string".to_string(),
            ServerAddrError::Other(err) => format!("unexpected error: {}", err),
        };

        write!(f, "{}", msg)
    }
}
