//! The visitor for variants, used to deserialize enums.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.
use serde::de::{DeserializeSeed, EnumVisitor, Visitor, Deserialize};
use serde::de::value::ValueDeserializer;

use de::Deserializer;

use error::*;
use defs::*;

pub struct VariantVisitor<'a, F: 'a + FnMut(&mut [u8]) -> Result<()>> {
    de: &'a mut Deserializer<F>,
    variants: &'static [&'static str],
}

impl<'a, F: FnMut(&mut [u8]) -> Result<()>> VariantVisitor<'a, F> {
    pub fn new(de: &'a mut Deserializer<F>,
               variants: &'static [&'static str])
               -> VariantVisitor<'a, F> {
        VariantVisitor {
            de: de,
            variants: variants,
        }
    }
}

impl<'a, F: FnMut(&mut [u8]) -> Result<()>> EnumVisitor for VariantVisitor<'a, F> {
    type Error = Error;
    type Variant = VariantVisitor<'a, F>;

    fn visit_variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant)>
        where V: DeserializeSeed
    {
        // get the variant index with a one-item tuple
        let variant_index_container: (usize, /* enum-type */) =
            Deserialize::deserialize(&mut *self.de)?;

        // the other value in this tuple would be the actual value of the enum,
        // but we don't know what that is
        let (variant_index /* enum-value */,) = variant_index_container;

        // translate that to the name of the variant
        let value = seed.deserialize(self.variants[variant_index].into_deserializer())?;

        Ok((value, self))
    }
}

impl<'a, F: FnMut(&mut [u8]) -> Result<()>> ::serde::de::VariantVisitor for VariantVisitor<'a, F> {
    type Error = Error;

    fn visit_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value>
        where V: Visitor
    {
        ::serde::Deserializer::deserialize(self.de, visitor)
    }

    fn visit_struct<V>(self, _: &'static [&'static str], visitor: V) -> Result<V::Value>
        where V: Visitor
    {
        ::serde::Deserializer::deserialize(self.de, visitor)
    }

    fn visit_newtype_seed<T>(self, seed: T) -> Result<T::Value>
        where T: DeserializeSeed
    {
        seed.deserialize(self.de)
    }

    fn visit_unit(self) -> Result<()> {
        Deserialize::deserialize(&mut *self.de)
    }
}
