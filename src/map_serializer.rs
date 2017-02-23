use collections::Vec;

use serde::ser::{Serialize, SerializeMap, SerializeStruct, SerializeStructVariant};

use byteorder::{ByteOrder, BigEndian};

use ser::Serializer;

use defs::*;
use error::*;

pub struct MapSerializer<'a, F: 'a + FnMut(&[u8]) -> Result<()>> {
    size: usize,
    buffer: Vec<u8>,
    output: &'a mut F,
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> MapSerializer<'a, F> {
    pub fn new(output: &'a mut F) -> MapSerializer<'a, F> {
        MapSerializer {
            size: 0,
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

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> SerializeMap for MapSerializer<'a, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        MapSerializer::serialize_element(self, key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        MapSerializer::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        MapSerializer::finish(self)
    }
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> SerializeStruct for MapSerializer<'a, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        MapSerializer::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<()> {
        MapSerializer::finish(self)
    }
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> SerializeStructVariant for MapSerializer<'a, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        MapSerializer::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<()> {
        MapSerializer::finish(self)
    }
}