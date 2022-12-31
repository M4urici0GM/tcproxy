use std::io::{Cursor, Read};
use bytes::Buf;
use crate::FrameDecodeError;

fn check_cursor_size<T>(src: &mut Cursor<&[u8]>) -> Result<(), FrameDecodeError>
    where
        T: Sized,
{
    if std::mem::size_of::<T>() > src.get_ref().len() - src.position() as usize {
        return Err(FrameDecodeError::Incomplete);
    }

    Ok(())
}

pub fn get_buffer(src: &mut Cursor<&[u8]>, buffer_size: u32) -> Result<Vec<u8>, FrameDecodeError> {
    let mut buffer = vec![0; buffer_size as usize];
    src.read_exact(&mut buffer).map_err(|_| FrameDecodeError::Incomplete)?;
    Ok(buffer)
}

pub fn get_u32(src: &mut Cursor<&[u8]>) -> Result<u32, FrameDecodeError> {
    check_cursor_size::<u32>(src)?;
    Ok(src.get_u32())
}

pub fn get_u16(src: &mut Cursor<&[u8]>) -> Result<u16, FrameDecodeError> {
    check_cursor_size::<u16>(src)?;
    Ok(src.get_u16())
}

pub fn get_u8(src: &mut Cursor<&[u8]>) -> Result<u8, FrameDecodeError> {
    check_cursor_size::<u8>(src)?;
    Ok(src.get_u8())
}

pub fn get_i64(src: &mut Cursor<&[u8]>) -> Result<i64, FrameDecodeError> {
    check_cursor_size::<i64>(src)?;
    Ok(src.get_i64())
}