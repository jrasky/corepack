//! A visitor for EXT items in a messagepack stream.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.
use collections::Vec;

use serde::de::{MapAccess, DeserializeSeed, IntoDeserializer, Error};

use defs::*;

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

impl<'a> MapAccess<'a> for ExtVisitor {
    type Error = ::serde::de::value::Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed<'a>
    {
        if self.state == 0 {
            let de = "type".into_deserializer();
            Ok(Some(try!(seed.deserialize(de))))
        } else if self.state == 1 {
            let de = "data".into_deserializer();
            Ok(Some(try!(seed.deserialize(de))))
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value>
        where T: DeserializeSeed<'a>
    {
        if self.state == 0 {
            self.state += 1;
            let de = self.ty.into_deserializer();
            Ok(try!(seed.deserialize(de)))
        } else if self.state == 1 {
            self.state += 1;
            let de = self.data.clone().into_deserializer();
            Ok(try!(seed.deserialize(de)))
        } else {
            Err(Error::custom("End of stream"))
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(2 - self.state as usize)
    }
}
