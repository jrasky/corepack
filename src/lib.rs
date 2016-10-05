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
// always test with libstd turned on
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#[cfg(all(not(feature = "std"), not(test)))]
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

mod defs;
pub mod error;
pub mod ser;
pub mod de;

// include serde generated code
include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

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

#[cfg(test)]
mod test {
    use serde::{Serialize, Deserialize};
    use std::fmt::Debug;

    use ::test_types::T;
    // #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    // enum T {
    //     A(usize),
    //     B,
    //     C(i8, i8),
    //     D { a: isize },
    // }

    fn test_through<T>(expected: T)
        where T: Serialize + Deserialize + PartialEq + Debug {
        let x = ::to_bytes(&expected).expect("Failed to serialize expected");

        let actual = ::from_bytes(&x).expect("Failed to deserialize expected");

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_str() {
        test_through(format!("Hello World!"))
    }

    #[test]
    fn test_enum() {
        test_through(T::B)
    }

    #[test]
    fn test_enum_newtype() {
        test_through(T::A(42))
    }

    #[test]
    fn test_enum_tuple() {
        test_through(T::C(-3, 22))
    }

    #[test]
    fn test_enum_struct() {
        test_through(T::D { a: 9001 })
    }
}
