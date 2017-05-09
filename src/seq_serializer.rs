//! The sequence serializer that formats sequences in messagepack.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.
use collections::Vec;

use serde::ser::{Serialize, SerializeSeq, SerializeTupleVariant, SerializeTuple,
                 SerializeTupleStruct, Error};

use byteorder::{ByteOrder, BigEndian};

use ser::Serializer;

use defs::*;

pub struct SeqSerializer<'a, F: 'a + FnMut(&[u8]) -> Result<()>> {
    count: usize,
    size: Option<usize>,
    buffer: Vec<u8>,
    output: &'a mut F,
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> SeqSerializer<'a, F> {
    pub fn new(output: &'a mut F) -> SeqSerializer<'a, F> {
        SeqSerializer {
            count: 0,
            size: None,
            buffer: vec![],
            output: output,
        }
    }

    pub fn hint_size(&mut self, size: Option<usize>) -> Result<()> {
        self.size = size;

        if let Some(size) = self.size {
            // output this now because we know it
            self.output_sequence_header(size)
        } else {
            Ok(())
        }
    }

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        self.count += 1;

        if self.should_serialize_directly() {
            self.serialize_directly(value)
        } else {
            self.serialize_into_buffer(value)
        }
    }

    fn finish(mut self) -> Result<()> {
        if let Some(size) = self.size {
            self.check_item_count_matches_size(size)?;
            Ok(())
        } else {
            let count = self.count;
            self.output_sequence_header(count)?;
            (self.output)(self.buffer.as_slice())
        }
    }

    fn check_item_count_matches_size(&self, size: usize) -> Result<()> {
        if size != self.count {
            Err(Error::custom("Bad length"))
        } else {
            Ok(())
        }
    }

    fn should_serialize_directly(&mut self) -> bool {
        self.size.is_some()
    }

    fn serialize_into_buffer<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        let mut target = Serializer::new(|bytes| {
            self.buffer.extend_from_slice(bytes);
            Ok(())
        });

        value.serialize(&mut target)
    }

    fn serialize_directly<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        let mut target = Serializer::new(|bytes| (self.output)(bytes));

        value.serialize(&mut target)
    }

    fn output_sequence_header(&mut self, size: usize) -> Result<()> {
        if size <= MAX_FIXARRAY {
            (self.output)(&[size as u8 | FIXARRAY_MASK])
        } else if size <= MAX_ARRAY16 {
            let mut buf = [ARRAY16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], size as u16);
            (self.output)(&buf)
        } else if size <= MAX_ARRAY32 {
            let mut buf = [ARRAY32; U32_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], size as u32);
            (self.output)(&buf)
        } else {
            Err(Error::custom("Too big"))
        }
    }
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> SerializeSeq for SeqSerializer<'a, F> {
    type Ok = ();
    type Error = ::serde::de::value::Error;

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
    type Error = ::serde::de::value::Error;

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
    type Error = ::serde::de::value::Error;

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
    type Error = ::serde::de::value::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
        where T: ?Sized + Serialize
    {
        SeqSerializer::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        SeqSerializer::finish(self)
    }
}
