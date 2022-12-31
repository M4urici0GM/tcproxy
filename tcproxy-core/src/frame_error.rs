use std::{num::TryFromIntError, string::FromUtf8Error};

#[derive(Debug)]
pub enum FrameDecodeError {
    Incomplete,
    Other(crate::Error),
}

impl From<String> for FrameDecodeError {
    fn from(src: String) -> FrameDecodeError {
        FrameDecodeError::Other(src.into())
    }
}

impl From<&str> for FrameDecodeError {
    fn from(src: &str) -> FrameDecodeError {
        FrameDecodeError::Other(src.into())
    }
}

impl std::error::Error for FrameDecodeError {}

impl From<FromUtf8Error> for FrameDecodeError {
    fn from(_src: FromUtf8Error) -> FrameDecodeError {
        "protocol error; invalid frame format".into()
    }
}

impl From<TryFromIntError> for FrameDecodeError {
    fn from(_src: TryFromIntError) -> FrameDecodeError {
        "protocol error; invalid frame format".into()
    }
}

impl std::fmt::Display for FrameDecodeError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FrameDecodeError::Incomplete => "stream ended early".fmt(fmt),
            FrameDecodeError::Other(err) => err.fmt(fmt),
        }
    }
}
