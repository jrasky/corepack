#![feature(inclusive_range)]
#![feature(inclusive_range_syntax)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(collections)]
#![feature(alloc)]
#![feature(range_contains)]
#![feature(const_fn)]
#![feature(box_syntax)]
#![allow(overflowing_literals)]
#![no_std]
extern crate core as std;
extern crate serde;
extern crate byteorder;
#[macro_use]
extern crate collections;
extern crate alloc;

use collections::Vec;

use std::ptr;

pub use ser::Serializer;
pub use de::Deserializer;
pub use generic::Generic;

mod defs;
pub mod generic;
pub mod error;
pub mod ser;
pub mod de;

pub fn from_iter<I, V>(mut iter: I) -> Result<V, error::Error>
    where I: Iterator<Item=u8>, V: serde::Deserialize {
    let mut de = Deserializer::new(|buf: &mut [u8]| {
        for i in 0..buf.len() {
            if let Some(byte) = iter.next() {
                buf[i] = byte;
            } else {
                return Err(error::Error::simple(error::Reason::EndOfStream));
            }
        }

        Ok(())
    });

    V::deserialize(&mut de)
}

pub fn from_bytes<V>(bytes: &[u8]) -> Result<V, error::Error>
    where V: serde::Deserialize {
    let mut position: usize = 0;

    let mut de = Deserializer::new(|buf: &mut [u8]| {
        if position + buf.len() > bytes.len() {
            Err(error::Error::simple(error::Reason::EndOfStream))
        } else {
            unsafe {
                ptr::copy(bytes.as_ptr().offset(position as isize), buf.as_mut_ptr(), buf.len());
            }

            position += buf.len();
            Ok(())
        }
    });

    V::deserialize(&mut de)
}

pub fn to_bytes<V>(value: V) -> Result<Vec<u8>, error::Error>
    where V: serde::Serialize {
    let mut bytes = vec![];

    {
        let mut ser = Serializer::new(|buf| {
            bytes.extend_from_slice(buf);
            Ok(())
        });

        try!(value.serialize(&mut ser));
    }

    Ok(bytes)
}

pub fn from_iter_generic<I>(mut iter: I) -> Result<Generic, error::Error>
    where I: Iterator<Item=u8> {
    let mut de = Deserializer::new(|buf: &mut [u8]| {
        for i in 0..buf.len() {
            if let Some(byte) = iter.next() {
                buf[i] = byte;
            } else {
                return Err(error::Error::simple(error::Reason::EndOfStream));
            }
        }

        Ok(())
    });

    de.deserialize_generic()
}

pub fn from_bytes_generic(bytes: &[u8]) -> Result<Generic, error::Error> {
    let mut position: usize = 0;

    let mut de = Deserializer::new(|buf: &mut [u8]| {
        if position + buf.len() > bytes.len() {
            Err(error::Error::simple(error::Reason::EndOfStream))
        } else {
            unsafe {
                ptr::copy(bytes.as_ptr().offset(position as isize), buf.as_mut_ptr(), buf.len());
            }

            position += buf.len();
            Ok(())
        }
    });

    de.deserialize_generic()
}

pub fn to_bytes_generic(value: &Generic) -> Result<Vec<u8>, error::Error> {
    let mut bytes = vec![];

    {
        let mut ser = Serializer::new(|buf| {
            bytes.extend_from_slice(buf);
            Ok(())
        });

        try!(value.serialize_pack(&mut ser));
    }

    Ok(bytes)
}
