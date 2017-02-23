//! A visitor for EXT items in a messagepack stream.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.
use collections::Vec;

use serde::de::{MapVisitor, DeserializeSeed};
use serde::de::value::ValueDeserializer;
use serde::bytes::Bytes;

use defs::*;
use error::*;

pub struct ExtVisitor {
    state: u8,
    ty: i8,
    data: Vec<u8>,
}

impl ExtVisitor {
    pub fn new(ty: i8, data: Vec<u8>) -> ExtVisitor {
        ExtVisitor {
            state: 0,
            ty: ty,
            data: data,
        }
    }
}

impl MapVisitor for ExtVisitor {
    type Error = Error;

    fn visit_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed
    {
        if self.state == 0 {
            let de = ValueDeserializer::<Error>::into_deserializer("type");
            Ok(Some(try!(seed.deserialize(de))))
        } else if self.state == 1 {
            let de = ValueDeserializer::<Error>::into_deserializer("data");
            Ok(Some(try!(seed.deserialize(de))))
        } else {
            Ok(None)
        }
    }

    fn visit_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
        where T: DeserializeSeed
    {
        if self.state == 0 {
            self.state += 1;
            let de = ValueDeserializer::<Error>::into_deserializer(self.ty);
            Ok(try!(seed.deserialize(de)))
        } else if self.state == 1 {
            self.state += 1;
            let de = ValueDeserializer::<Error>::into_deserializer(Bytes::from(self.data
                .as_slice()));
            Ok(try!(seed.deserialize(de)))
        } else {
            Err(Error::simple(Reason::EndOfStream))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (2 - self.state as usize, Some(2 - self.state as usize))
    }
}
