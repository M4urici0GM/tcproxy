use std::{num::TryFromIntError, string::FromUtf8Error};
use std::io::Error;

#[derive(Debug)]
pub enum FrameDecodeError {
    Incomplete,
    UnexpectedFrameType(u8),
    CorruptedFrame,
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

impl From<std::io::Error> for FrameDecodeError {
    fn from(value: Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::UnexpectedEof => Self::Incomplete,
            _ => Self::Other(value.into())
        }
    }
}

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
            FrameDecodeError::CorruptedFrame => "frame is corrupted".fmt(fmt),
            FrameDecodeError::Incomplete => "stream ended early".fmt(fmt),
            FrameDecodeError::Other(err) => err.fmt(fmt),
            FrameDecodeError::UnexpectedFrameType(f_type) => format!("unexpected frame_type: {}", f_type).fmt(fmt),
        }
    }
}
