use collections::Vec;

use serde::ser::{Serialize, SerializeSeq, SerializeTupleVariant, SerializeTuple,
                 SerializeTupleStruct};

use byteorder::{ByteOrder, BigEndian};

use ser::Serializer;

use defs::*;
use error::*;

pub struct SeqSerializer<'a, F: 'a + FnMut(&[u8]) -> Result<()>> {
    size: usize,
    buffer: Vec<u8>,
    output: &'a mut F,
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> SeqSerializer<'a, F> {
    pub fn new(output: &'a mut F) -> SeqSerializer<'a, F> {
        SeqSerializer {
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
        if self.size <= MAX_FIXARRAY {
            try!((self.output)(&[self.size as u8 | FIXARRAY_MASK]));
        } else if self.size <= MAX_ARRAY16 {
            let mut buf = [ARRAY16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], self.size as u16);
            try!((self.output)(&buf));
        } else if self.size <= MAX_ARRAY32 {
            let mut buf = [ARRAY32; U32_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], self.size as u32);
            try!((self.output)(&buf));
        } else {
            return Err(Error::simple(Reason::TooBig));
        }

        (self.output)(self.buffer.as_slice())
    }
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> SerializeSeq for SeqSerializer<'a, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SeqSerializer::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        SeqSerializer::finish(self)
    }
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> SerializeTupleVariant for SeqSerializer<'a, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SeqSerializer::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        SeqSerializer::finish(self)
    }
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> SerializeTupleStruct for SeqSerializer<'a, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SeqSerializer::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        SeqSerializer::finish(self)
    }
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> SerializeTuple for SeqSerializer<'a, F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SeqSerializer::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        SeqSerializer::finish(self)
    }
}