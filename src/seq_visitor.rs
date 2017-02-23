//! The visitor that decodes sequences.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.
use serde::de::{MapVisitor, DeserializeSeed};

use de::Deserializer;

use defs::*;
use error::*;

pub struct SeqVisitor<'a, F: 'a + FnMut(&mut [u8]) -> Result<()>> {
    de: &'a mut Deserializer<F>,
    count: usize,
}

impl<'a, F: FnMut(&mut [u8]) -> Result<()>> SeqVisitor<'a, F> {
    pub fn new(de: &'a mut Deserializer<F>, count: usize) -> SeqVisitor<'a, F> {
        SeqVisitor {
            de: de,
            count: count,
        }
    }

    fn visit_item<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed
    {
        if self.count == 0 {
            return Ok(None);
        }

        self.count -= 1;

        Ok(Some(try!(seed.deserialize(&mut *self.de))))
    }
}

impl<'a, F: FnMut(&mut [u8]) -> Result<()>> ::serde::de::SeqVisitor for SeqVisitor<'a, F> {
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed
    {
        self.visit_item(seed)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl<'a, F: FnMut(&mut [u8]) -> Result<()>> MapVisitor for SeqVisitor<'a, F> {
    type Error = Error;

    fn visit_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: DeserializeSeed
    {
        self.visit_item(seed)
    }

    fn visit_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: DeserializeSeed
    {
        self.visit_item(seed)
            .and_then(|maybe_value| maybe_value.ok_or(Error::simple(Reason::EndOfStream)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count / 2, Some((self.count + 1) / 2))
    }
}