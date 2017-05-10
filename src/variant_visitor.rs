//! The visitor for variants, used to deserialize enums.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.
use collections::borrow::ToOwned;

use serde::de::{IntoDeserializer, DeserializeSeed, EnumAccess, Visitor, Deserialize};
use serde::de::value::StringDeserializer;

use de::Deserializer;

use error::Error;

pub struct VariantVisitor<'de: 'a, 'a, F: 'a + FnMut(usize) -> Result<&'de [u8], Error>> {
    de: &'a mut Deserializer<'de, F>,
    variants: &'static [&'static str],
}

impl<'de, 'a, F: FnMut(usize) -> Result<&'de [u8], Error>> VariantVisitor<'de, 'a, F> {
    pub fn new(de: &'a mut Deserializer<'de, F>,
               variants: &'static [&'static str])
               -> VariantVisitor<'de, 'a, F> {
        VariantVisitor {
            de: de,
            variants: variants,
        }
    }
}

impl<'de, 'a, F: FnMut(usize) -> Result<&'de [u8], Error>> EnumAccess<'de> for VariantVisitor<'de, 'a, F> {
    type Error = Error;
    type Variant = VariantVisitor<'de, 'a, F>;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Error>
        where V: DeserializeSeed<'de>
    {
        // get the variant index with a one-item tuple
        let variant_index_container: (usize, /* enum-type */) =
            Deserialize::deserialize(&mut *self.de)?;

        // the other value in this tuple would be the actual value of the enum,
        // but we don't know what that is
        let (variant_index, /* enum-value */) = variant_index_container;

        // translate that to the name of the variant
        let name = self.variants[variant_index].to_owned();
        let de: StringDeserializer<Error> = name.into_deserializer();
        let value = seed.deserialize(de)?;

        Ok((value, self))
    }
}

impl<'de, 'a, F: FnMut(usize) -> Result<&'de [u8], Error>> ::serde::de::VariantAccess<'de> for VariantVisitor<'de, 'a, F> {
    type Error = Error;

    fn tuple_variant<V>(self, _: usize, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        ::serde::Deserializer::deserialize_any(self.de, visitor)
    }

    fn struct_variant<V>(self, _: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        ::serde::Deserializer::deserialize_any(self.de, visitor)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
        where T: DeserializeSeed<'de>
    {
        seed.deserialize(self.de)
    }

    fn unit_variant(self) -> Result<(), Error> {
        Deserialize::deserialize(&mut *self.de)
    }
}
