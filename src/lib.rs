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

pub use ser::Serializer;
pub use de::Deserializer;

mod defs;
pub mod error;
pub mod ser;
pub mod de;
