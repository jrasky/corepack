use collections::String;

use std::mem;

use byteorder::{ByteOrder, BigEndian};

use serde;

use defs::*;
use error::*;

pub struct Deserializer<F: FnMut(&mut [u8]) -> Result<(), Error>> {
    input: F
}

struct SeqVisitor<'a, F: 'a + FnMut(&mut [u8]) -> Result<(), Error>> {
    de: &'a mut Deserializer<F>,
    count: usize
}

impl<'a, F: FnMut(&mut [u8]) -> Result<(), Error>> SeqVisitor<'a, F> {
    fn new(de: &'a mut Deserializer<F>, count: usize) -> SeqVisitor<'a, F> {
        SeqVisitor {
            de: de,
            count: count
        }
    }
}

impl<'a, F: FnMut(&mut [u8]) -> Result<(), Error>> serde::de::SeqVisitor for SeqVisitor<'a, F> {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: serde::Deserialize {
        if self.count == 0 {
            return Ok(None);
        }

        self.count -= 1;

        Ok(Some(try!(T::deserialize(self.de))))
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.count != 0 {
            Err(Error::simple(Reason::ExtraItems))
        } else {
            Ok(())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count, Some(self.count))
    }
}

impl<'a, F: FnMut(&mut [u8]) -> Result<(), Error>> serde::de::MapVisitor for SeqVisitor<'a, F> {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, Error>
        where K: serde::Deserialize {
        serde::de::SeqVisitor::visit(self)
    }

    fn visit_value<V>(&mut self) -> Result<V, Error>
        where V: serde::Deserialize {
        try!(serde::de::SeqVisitor::visit(self)).ok_or(Error::simple(Reason::EndOfStream))
    }

    fn end(&mut self) -> Result<(), Error> {
        serde::de::SeqVisitor::end(self)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.count / 2, Some((self.count + 1) / 2))
    }
}

impl<F: FnMut(&mut [u8]) -> Result<(), Error>> Deserializer<F> {
    pub const fn new(input: F) -> Deserializer<F> {
        Deserializer {
            input: input
        }
    }

    fn input(&mut self, buf: &mut [u8]) -> Result<(), Error> {
        self.input.call_mut((buf,))
    }
}

impl<F: FnMut(&mut [u8]) -> Result<(), Error>> serde::Deserializer for Deserializer<F> {
    type Error = Error;

    fn deserialize<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        let mut buf = [0];
        try!(self.input(&mut buf));

        match buf[0] {
            v if POS_FIXINT.contains(v) => {
                visitor.visit_u8(v)
            }
            v if NEG_FIXINT.contains(v) => {
                visitor.visit_i8(unsafe {mem::transmute(v)})
            }
            v if FIXMAP.contains(v) => {
                let size = (buf[0] & !FIXMAP_MASK) as usize * 2;
                visitor.visit_map(SeqVisitor::new(self, size))
            }
            v if FIXARRAY.contains(v) => {
                let size = (buf[0] & !FIXARRAY_MASK) as usize;
                visitor.visit_seq(SeqVisitor::new(self, size))
            }
            v if FIXSTR.contains(v) => {
                let mut buf = vec![0; (buf[0] & !FIXSTR_MASK) as usize];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_string(
                    try!(String::from_utf8(buf)
                         .map_err(|e| Error::new(Reason::UTF8Error, format!("{}", e)))))
            }
            NIL => visitor.visit_none(),
            FALSE => visitor.visit_bool(false),
            TRUE => visitor.visit_bool(true),
            BIN8 => {
                let mut buf = [0];
                try!(self.input(&mut buf));
                let mut buf = vec![0; buf[0] as usize];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_byte_buf(buf)
            }
            BIN16 => {
                let mut buf = [0; U16_BYTES];
                try!(self.input(&mut buf));
                let mut buf = vec![0; BigEndian::read_u16(&buf) as usize];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_byte_buf(buf)
            }
            BIN32 => {
                let mut buf = [0; U32_BYTES];
                try!(self.input(&mut buf));
                let mut buf = vec![0; BigEndian::read_u32(&buf)as usize];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_byte_buf(buf)
            }
            EXT8 | EXT16 | EXT32 => {
                Err(Error::simple(Reason::BadType))
            }
            UINT8 => {
                let mut buf = [0];
                try!(self.input(&mut buf));
                visitor.visit_u8(buf[0])
            }
            UINT16 => {
                let mut buf = [0; U16_BYTES];
                try!(self.input(&mut buf));
                visitor.visit_u16(BigEndian::read_u16(&buf))
            }
            UINT32 => {
                let mut buf = [0; U32_BYTES];
                try!(self.input(&mut buf));
                visitor.visit_u32(BigEndian::read_u32(&buf))
            }
            UINT64 => {
                let mut buf = [0; U64_BYTES];
                try!(self.input(&mut buf));
                visitor.visit_u64(BigEndian::read_u64(&buf))
            }
            INT8 => {
                let mut buf = [0];
                try!(self.input(&mut buf));
                visitor.visit_i8(unsafe {mem::transmute(buf[0])})
            }
            INT16 => {
                let mut buf = [0; U16_BYTES];
                try!(self.input(&mut buf));
                visitor.visit_i16(BigEndian::read_i16(&buf))
            }
            INT32 => {
                let mut buf = [0; U32_BYTES];
                try!(self.input(&mut buf));
                visitor.visit_i32(BigEndian::read_i32(&buf))
            }
            INT64 => {
                let mut buf = [0; U64_BYTES];
                try!(self.input(&mut buf));
                visitor.visit_i64(BigEndian::read_i64(&buf))
            }
            FIXEXT1 | FIXEXT2 | FIXEXT4 | FIXEXT8 | FIXEXT16 => {
                Err(Error::simple(Reason::BadType))
            }
            STR8 => {
                let mut buf = [0];
                try!(self.input(&mut buf));
                let mut buf = vec![0; buf[0] as usize];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_string(
                    try!(String::from_utf8(buf)
                         .map_err(|e| Error::new(Reason::UTF8Error, format!("{}", e)))))
            }
            STR16 => {
                let mut buf = [0; U16_BYTES];
                try!(self.input(&mut buf));
                let mut buf = vec![0; BigEndian::read_u16(&buf) as usize];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_string(
                    try!(String::from_utf8(buf)
                         .map_err(|e| Error::new(Reason::UTF8Error, format!("{}", e)))))
            }
            STR32 => {
                let mut buf = [0; U32_BYTES];
                try!(self.input(&mut buf));
                let mut buf = vec![0; BigEndian::read_u32(&buf) as usize];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_string(
                    try!(String::from_utf8(buf)
                         .map_err(|e| Error::new(Reason::UTF8Error, format!("{}", e)))))
            }
            ARRAY16 => {
                let mut buf = [0; U16_BYTES];
                try!(self.input(&mut buf));
                let size = BigEndian::read_u16(&buf);
                visitor.visit_seq(SeqVisitor::new(self, size as usize))
            }
            ARRAY32 => {
                let mut buf = [0; U32_BYTES];
                try!(self.input(&mut buf));
                let size = BigEndian::read_u32(&buf);
                visitor.visit_seq(SeqVisitor::new(self, size as usize))
            }
            MAP16 => {
                let mut buf = [0; U16_BYTES];
                try!(self.input(&mut buf));
                let size = BigEndian::read_u16(&buf);
                visitor.visit_map(SeqVisitor::new(self, size as usize * 2))
            }
            MAP32 => {
                let mut buf = [0; U16_BYTES];
                try!(self.input(&mut buf));
                let size = BigEndian::read_u32(&buf);
                visitor.visit_map(SeqVisitor::new(self, size as usize * 2))
            }
            _ => {
                Err(Error::simple(Reason::BadType))
            }
        }
    }
}
