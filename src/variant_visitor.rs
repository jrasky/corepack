use serde::de::{SeqVisitor, DeserializeSeed, EnumVisitor, Visitor};

use de::Deserializer;

use error::*;
use defs::*;

pub struct VariantVisitor<'a, F: 'a + FnMut(&mut [u8]) -> Result<()>> {
    de: &'a mut Deserializer<F>,
    count: usize,
}

impl<'a, F: FnMut(&mut [u8]) -> Result<()>> VariantVisitor<'a, F> {
    pub fn new(de: &'a mut Deserializer<F>, count: usize) -> VariantVisitor<'a, F> {
        VariantVisitor {
            de: de,
            count: count
        }
    }
}

impl<'a, F: FnMut(&mut [u8]) -> Result<()>> SeqVisitor for VariantVisitor<'a, F> {
    type Error = Error;

    fn visit_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed
    {
        if self.count == 0 {
            return Ok(None);
        }

        self.count -= 1;

        let value = seed.deserialize(&mut *self.de)?;

        Ok(Some(value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl<'a, F: FnMut(&mut [u8]) -> Result<()>> EnumVisitor
    for VariantVisitor<'a, F> {
    type Error = Error;
    type Variant = VariantVisitor<'a, F>;

    fn visit_variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
        where V: DeserializeSeed
    {
        let value = seed.deserialize(&mut *self.de)?;

        Ok((value, self))
    }
}

impl<'a, F: FnMut(&mut [u8]) -> Result<()>> ::serde::de::VariantVisitor
    for
    VariantVisitor<'a, F> {
    type Error = Error;

    fn visit_tuple<V>(self, _: usize, mut visitor: V) -> Result<V::Value>
        where V: Visitor
    {
        // tuple variants have an extra item added to them
        visitor.visit_seq(self)
    }

    fn visit_struct<V>(mut self, _: &'static [&'static str], visitor: V) -> Result<V::Value>
        where V: Visitor
    {
        // struct variants are encoded as a tuple with the discriminant and then the encoded struct
        // so the encoded struct should just be the next element
        if self.count == 0 {
            return Err(Error::simple(Reason::EndOfStream));
        }

        self.count -= 1;

        // universal function call syntax because I'm lazy
        ::serde::Deserializer::deserialize(self.de, visitor)
    }

    fn visit_newtype_seed<T>(mut self, seed: T) -> Result<T::Value>
        where T: DeserializeSeed
    {
        // newtypes are encoded as two-element tuples
        if self.count == 0 {
            return Err(Error::simple(Reason::EndOfStream));
        }

        self.count -= 1;
        seed.deserialize(self.de)
    }

    fn visit_unit(self) -> Result<()> {
        Ok(())
    }
}