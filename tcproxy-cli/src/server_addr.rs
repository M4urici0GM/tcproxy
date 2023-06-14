use fancy_regex::Regex;
use lazy_static::lazy_static;
use std::error::Error;
use std::net::{AddrParseError, IpAddr, SocketAddr};
use std::{fmt::Display, num::ParseIntError, str::FromStr};

use crate::config::AppContext;

lazy_static! {
    static ref IP_REGEX: Regex =
        Regex::new(r"^((25[0-5]|(2[0-4]|1\d|[1-9]|)\d)\.?\b){4}$").unwrap();
}

#[derive(Debug)]
pub enum ServerAddrError {
    InvalidString,
    InvalidPort,
    InvalidHost,
    Other(tcproxy_core::Error),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ServerAddrType {
    IpAddr,
    Dns,
}

#[derive(Debug, Clone)]
pub struct ServerAddr {
    host: String,
    port: u16,
    addr_type: ServerAddrType,
}

impl TryFrom<AppContext> for ServerAddr {
    type Error = ServerAddrError;

    fn try_from(value: AppContext) -> Result<Self, Self::Error> {
        Self::new(value.host(), value.port())
    }
}

// Used to represent a ServerAddr
// It will be useful for allowing app contexts to store domain names, instead of only IPs
impl ServerAddr {
    pub fn new(host: &str, port: &u16) -> Result<Self, ServerAddrError> {
        if host == String::default() {
            return Err(ServerAddrError::InvalidHost);
        }

        Ok(Self {
            host: host.to_owned(),
            port: port.to_owned(),
            addr_type: Self::parse_type(host)?,
        })
    }

    fn parse_type(host: &str) -> Result<ServerAddrType, ServerAddrError> {
        match Self::is_ip(host)? {
            true => Ok(ServerAddrType::IpAddr),
            _ => Ok(ServerAddrType::Dns),
        }
    }

    fn is_ip(host: &str) -> Result<bool, ServerAddrError> {
        Ok(IP_REGEX.is_match(host)?)
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

    pub fn to_socket_addr(&self) -> Result<SocketAddr, ServerAddrError> {
        let ip = IpAddr::from_str(&self.host)?;
        let addr = SocketAddr::new(ip, self.port);

        Ok(addr)
    }
}

impl FromStr for ServerAddr {
    type Err = ServerAddrError;

    fn from_str(given_str: &str) -> Result<Self, Self::Err> {
        let groups: Vec<&str> = given_str.split(':').collect();
        if 2 != groups.len() {
            return Err(ServerAddrError::InvalidString);
        }

        let port = match groups[1].parse::<u16>() {
            Ok(port) => port,
            Err(_) => return Err(ServerAddrError::InvalidPort),
        };

        ServerAddr::new(groups[0], &port)
    }
}

impl Error for ServerAddrError {}

impl From<AddrParseError> for ServerAddrError {
    fn from(_: AddrParseError) -> Self {
        Self::InvalidHost
    }
}

impl From<ParseIntError> for ServerAddrError {
    fn from(err: ParseIntError) -> Self {
        Self::Other(err.into())
    }
}

impl From<fancy_regex::Error> for ServerAddrError {
    fn from(err: fancy_regex::Error) -> Self {
        Self::Other(err.into())
    }
}

impl Display for ServerAddrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = match self {
            ServerAddrError::InvalidPort => "invalid port.".to_string(),
            ServerAddrError::InvalidString => "invalid host string should be IP:PORT".to_string(),
            ServerAddrError::InvalidHost => "invalid host".to_string(),
            ServerAddrError::Other(err) => format!("unexpected error: {}", err),
        };

        write!(f, "{}", msg)
    }
}

#[cfg(test)]
mod tests {
    use crate::server_addr::{ServerAddr, ServerAddrError, ServerAddrType};
    use std::str::FromStr;
    use tcproxy_core::is_type;

    #[test]
    pub fn should_recognize_ip_addr() {
        // Arrange
        let server_addr = ServerAddr::new("127.0.0.1", &8080).unwrap();

        // Assert
        assert_eq!(server_addr.addr_type, ServerAddrType::IpAddr);
    }

    #[test]
    pub fn should_recognize_domain_name() {
        // Arrange
        let server_addr = ServerAddr::new("localhost", &8080).unwrap();

        // Assert
        assert_eq!(server_addr.addr_type, ServerAddrType::Dns);
    }

    #[test]
    pub fn should_return_err_when_host_is_invalid() {
        // Arrange
        let test_str = ":8080";

        // Act
        let result = ServerAddr::from_str(test_str);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), ServerAddrError::InvalidHost));
    }

    #[test]
    pub fn should_return_err_when_str_is_invalid() {
        // Arrange
        let test_str = "some_invalid_str";

        // Act
        let result = ServerAddr::from_str(test_str);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(
            result.unwrap_err(),
            ServerAddrError::InvalidString
        ));
    }

    #[test]
    pub fn should_return_err_when_port_is_invalid() {
        // Arrange
        let test_str = ":test_port";

        // Act
        let result = ServerAddr::from_str(test_str);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), ServerAddrError::InvalidPort));
    }

    #[test]
    pub fn should_return_err_when_port_is_greater_than_limit() {
        // Arrange
        let test_str = ":66000";

        // Act
        let result = ServerAddr::from_str(test_str);

        // Assert
        assert!(result.is_err());
        assert!(is_type!(result.unwrap_err(), ServerAddrError::InvalidPort));
    }

    #[test]
    pub fn should_be_able_to_parse_from_str() {
        // Arrange
        let ip_addr = "127.0.0.1:8080";

        // Act
        let server_addr = ServerAddr::from_str(ip_addr).unwrap();

        // Assert
        assert_eq!("127.0.0.1", server_addr.host());
        assert_eq!(8080u16, *server_addr.port());
    }
}
