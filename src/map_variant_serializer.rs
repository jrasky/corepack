use collections::Vec;

use serde::ser::{Serialize, SerializeSeq, SerializeMap, SerializeTupleVariant, SerializeStruct,
                 SerializeTuple, SerializeTupleStruct, SerializeStructVariant};

use byteorder::{ByteOrder, BigEndian, LittleEndian};

use ser::Serializer;

use defs::*;
use error::*;
use seq_serializer::*;

pub struct MapVariantSerializer<'a, F: 'a + FnMut(&[u8]) -> Result<()>> {
    size: usize,
    variant: usize,
    buffer: Vec<u8>,
    output: &'a mut F,
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> MapVariantSerializer<'a, F> {
    pub fn new(variant: usize, output: &'a mut F) -> MapVariantSerializer<'a, F> {
        MapVariantSerializer {
            size: 0,
            variant: variant,
            buffer: vec![],
            output: output,
        }
    }

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        self.size += 1;

        let mut target = Serializer::new(|bytes| {
            self.buffer.extend_from_slice(bytes);
            Ok(())
        });

        value.serialize(&mut target)
    }

    fn finish(mut self) -> Result<()> {
        // get the serialized variant
        let variant_buffer = self.serialize_variant()?;

        // output a tuple of size two
        (self.output)(&[2u8 | FIXARRAY_MASK])?;

        // output our variant index
        (self.output)(&*variant_buffer)?;

        // now output the map as the second element
        self.output_map()
    }

    fn serialize_variant(&self) -> Result<Vec<u8>> {
        // create a temporary buffer for the tuple and serialize the variant index
        let mut buffer = vec![];

        {
            let mut target = Serializer::new(|bytes: &[u8]| {
                buffer.extend_from_slice(bytes);
                Ok(())
            });

            self.variant.serialize(&mut target)?;
        }

        Ok(buffer)
    }

    fn output_map(&mut self) -> Result<()> {
        if self.size % 1 != 0 {
            return Err(Error::simple(Reason::BadLength));
        }

        self.size /= 2;

        if self.size <= MAX_FIXMAP {
            try!((self.output)(&[self.size as u8 | FIXMAP_MASK]));
        } else if self.size <= MAX_MAP16 {
            let mut buf = [MAP16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], self.size as u16);
            try!((self.output)(&buf));
        } else if self.size <= MAX_MAP32 {
            let mut buf = [MAP32; U32_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], self.size as u32);
            try!((self.output)(&buf));
        } else {
            return Err(Error::simple(Reason::TooBig));
        }

        (self.output)(self.buffer.as_slice())
    }
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> SerializeStructVariant
    for MapVariantSerializer<'a, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        MapVariantSerializer::serialize_element(self, key)?;
        MapVariantSerializer::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        MapVariantSerializer::finish(self)
    }
}