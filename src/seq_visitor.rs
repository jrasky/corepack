//! The visitor that decodes sequences.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.
use serde::de::{MapAccess, DeserializeSeed};

use de::Deserializer;

use error::Error;

pub struct SeqVisitor<'de: 'a, 'a, F: 'a + FnMut(usize) -> Result<&'de [u8], Error>> {
    de: &'a mut Deserializer<'de, F>,
    count: usize,
}

impl<'de, 'a, F: FnMut(usize) -> Result<&'de [u8], Error>> SeqVisitor<'de, 'a, F> {
    pub fn new(de: &'a mut Deserializer<'de, F>, count: usize) -> SeqVisitor<'de, 'a, F> {
        SeqVisitor {
            de: de,
            count: count,
        }
    }

    fn visit_item<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed<'de>
    {
        if self.count == 0 {
            return Ok(None);
        }

        self.count -= 1;

        Ok(Some(try!(seed.deserialize(&mut *self.de))))
    }
}

impl<'de, 'a, F: FnMut(usize) -> Result<&'de [u8], Error>> ::serde::de::SeqAccess<'de> for SeqVisitor<'de, 'a, F> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
        where T: DeserializeSeed<'de>
    {
        self.visit_item(seed)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.count)
    }
}

impl<'de, 'a, F: FnMut(usize) -> Result<&'de [u8], Error>> MapAccess<'de> for SeqVisitor<'de, 'a, F> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
        where K: DeserializeSeed<'de>
    {
        self.visit_item(seed)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
        where V: DeserializeSeed<'de>
    {
        self.visit_item(seed)
            .and_then(|maybe_value| maybe_value.ok_or(Error::EndOfStream))
    }

    fn size_hint(&self) -> Option<usize> {
        Some((self.count + 1) / 2)
    }
}
