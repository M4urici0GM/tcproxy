
#[cfg(test)]
pub fn generate_random_buffer(buffer_size: i32) -> bytes::BytesMut {
    use bytes::{BufMut, BytesMut};

    let initial_vec: Vec<u8> = vec![];
    let result = (0..buffer_size)
        .map(|_| rand::random::<u8>())
        .fold(initial_vec, |mut a, b| {
            a.put_u8(b);
            a
        });

    BytesMut::from(result.as_slice())
}

#[macro_export]
macro_rules! extract_enum_value {
  ($value:expr, $pattern:pat => $extracted_value:expr) => {
    match $value {
      $pattern => $extracted_value,
      _ => panic!("Pattern doesn't match!"),
    }
  };
}
