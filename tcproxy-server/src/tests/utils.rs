use bytes::{BytesMut, BufMut};
use std;
use proc_macro2::TokenStream;
use syn::{parse_macro_input, DeriveInput, FieldsNamed, Type};
use syn::__private::quote::quote;
use syn::__private::TokenStream2;


pub fn generate_random_buffer(buffer_size: i32) -> BytesMut {
  let mut buffer = BytesMut::with_capacity(buffer_size as usize);

  (0..buffer_size)
      .for_each(|_| {
          let random = rand::random::<u8>();
          buffer.put_u8(random);
      });

  return buffer;
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
