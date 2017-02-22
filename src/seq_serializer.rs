use collections::Vec;

use serde::ser::{Serialize, SerializeSeq, SerializeMap, SerializeTupleVariant, SerializeStruct,
                 SerializeTuple, SerializeTupleStruct};

use byteorder::{ByteOrder, BigEndian, LittleEndian};

use ser::Serializer;

use defs::*;
use error::*;

// TODO: resolve this hack
pub struct SeqSerializer<F: FnMut(&[u8]) -> Result<()>> {
    pub size: usize,
    buffer: Vec<u8>,
    pub output: F,
}

impl<F: FnMut(&[u8]) -> Result<()>> SeqSerializer<F> {
    pub fn new(output: F) -> SeqSerializer<F> {
        SeqSerializer {
            size: 0,
            buffer: vec![],
            output: output,
        }
    }
}

impl<F: FnMut(&[u8]) -> Result<()>> SerializeMap for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SerializeSeq::serialize_element(self, key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(mut self) -> Result<()> {
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

impl<F: FnMut(&[u8]) -> Result<()>> SerializeStruct for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<()> {
        SerializeSeq::end(self)
    }
}

impl<F: FnMut(&[u8]) -> Result<()>> SerializeTupleVariant for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        SerializeStruct::end(self)
    }
}

impl<F: FnMut(&[u8]) -> Result<()>> SerializeTupleStruct for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        SerializeTuple::end(self)
    }
}

impl<F: FnMut(&[u8]) -> Result<()>> SerializeTuple for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        SerializeSeq::end(self)
    }
}

impl<F: FnMut(&[u8]) -> Result<()>> SerializeSeq for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        self.size += 1;

        let target = Serializer::new(|bytes| {
            self.buffer.extend_from_slice(bytes);
            Ok(())
        });

        value.serialize(target)
    }

    fn end(mut self) -> Result<()> {
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