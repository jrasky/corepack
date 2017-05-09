//! The visitor that decodes sequences.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.
use serde::de::{MapAccess, DeserializeSeed};

use de::Deserializer;

use error::Error;

pub struct SeqVisitor<'a, F: 'a + FnMut(&mut [u8]) -> Result<(), Error>> {
    de: &'a mut Deserializer<F>,
    count: usize,
}

impl<'a, F: FnMut(&mut [u8]) -> Result<(), Error>> SeqVisitor<'a, F> {
    pub fn new(de: &'a mut Deserializer<F>, count: usize) -> SeqVisitor<'a, F> {
        SeqVisitor {
            de: de,
            count: count,
        }
    }

    fn visit_item<'b, T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed<'b>
    {
        if self.count == 0 {
            return Ok(None);
        }

        self.count -= 1;

        Ok(Some(try!(seed.deserialize(&mut *self.de))))
    }
}

impl<'a, 'b, F: FnMut(&mut [u8]) -> Result<(), Error>> ::serde::de::SeqAccess<'a> for SeqVisitor<'b, F> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed<'a>
    {
        self.visit_item(seed)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.count)
    }
}

impl<'a, 'b, F: FnMut(&mut [u8]) -> Result<(), Error>> MapAccess<'a> for SeqVisitor<'b, F> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where K: DeserializeSeed<'a>
    {
        self.visit_item(seed)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where V: DeserializeSeed<'a>
    {
        self.visit_item(seed)
            .and_then(|maybe_value| maybe_value.ok_or(Error::EndOfStream))
    }

    fn size_hint(&self) -> Option<usize> {
        Some((self.count + 1) / 2)
    }
}
