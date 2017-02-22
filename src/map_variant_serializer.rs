use collections::Vec;

use serde::ser::{Serialize, SerializeSeq, SerializeMap, SerializeTupleVariant, SerializeStruct,
                 SerializeTuple, SerializeTupleStruct, SerializeStructVariant};

use byteorder::{ByteOrder, BigEndian, LittleEndian};

use defs::*;
use error::*;
use seq_serializer::*;

pub struct MapVariantSerializer<F: FnMut(&[u8]) -> Result<()>> {
    map_container: SeqSerializer<F>,
    variant: usize,
}

impl<F: FnMut(&[u8]) -> Result<()>> MapVariantSerializer<F> {
    pub fn new(variant: usize, output: F) -> MapVariantSerializer<F> {
        let mut map_container = SeqSerializer::new(output);

        MapVariantSerializer {
            map_container: map_container,
            variant: variant,
        }
    }
}

impl<F: FnMut(&[u8]) -> Result<()>> SerializeStructVariant for MapVariantSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SerializeMap::serialize_entry(&mut self.map_container, key, value)
    }

    fn end(mut self) -> Result<()> {
        // the next part is entirely a hack, and really should be reworked. Because of the structure of messagepack,
        // we can just keep encoding things without worry. What we do is steal the output from the map container, and
        // use it to encode a two-element tuple containing the variant, and then give the output back to the map
        // serializer and finish everything up. The end result is a two-element tuple containing the variant and the
        // map containing the actual struct fields, but man is this a hack.

        // TODO: rework this to be a bit less hackey

        // create a buffer for our trouble
        let mut buffer: Vec<u8> = vec![];
        {
            let output = |buf: &[u8]| {
                buffer.extend_from_slice(buf);
                Ok(())
            };

            let mut variant_container = SeqSerializer::new(output);
            SerializeTuple::serialize_element(&mut variant_container, &self.variant);

            variant_container.size = 2;

            // write out the variant container
            SerializeSeq::end(variant_container)?;
        }

        // output the buffer
        (self.map_container.output)(buffer.as_slice())?;

        // then write out the map
        SerializeMap::end(self.map_container)
    }
}