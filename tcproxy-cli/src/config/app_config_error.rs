use tcproxy_core::Error;

#[derive(Debug)]
pub enum AppConfigError {
    NotFound,
    YamlErr(serde_yaml::Error),
    IOError(std::io::Error),
    Other(Error)
}

impl From<serde_yaml::Error> for AppConfigError {
    fn from(err: serde_yaml::Error) -> Self {
        AppConfigError::YamlErr(err)
    }
}

impl From<std::io::Error> for AppConfigError {
    fn from(err: std::io::Error) -> Self {
        AppConfigError::IOError(err)
    }
}

impl From<String> for AppConfigError {
    fn from(msg: String) -> Self {
        AppConfigError::Other(msg.into())
    }
}

impl From<&str> for AppConfigError {
    fn from(msg: &str) -> Self {
        AppConfigError::Other(msg.into())
    }
}