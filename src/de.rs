use collections::{String, Vec};

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

struct ExtVisitor {
    state: u8,
    ty: i8,
    data: Vec<u8>
}

impl<'a, F: FnMut(&mut [u8]) -> Result<(), Error>> SeqVisitor<'a, F> {
    fn new(de: &'a mut Deserializer<F>, count: usize) -> SeqVisitor<'a, F> {
        SeqVisitor {
            de: de,
            count: count
        }
    }
}

impl serde::de::MapVisitor for ExtVisitor {
    type Error = Error;

    fn visit_key<T>(&mut self) -> Result<Option<T>, Error> where T: serde::Deserialize {
        if self.state == 0 {
            let mut de = serde::de::value::ValueDeserializer::<Error>::into_deserializer("type");
            Ok(Some(try!(T::deserialize(&mut de))))
        } else if self.state == 1 {
            let mut de = serde::de::value::ValueDeserializer::<Error>::into_deserializer("data");
            Ok(Some(try!(T::deserialize(&mut de))))
        } else {
            Ok(None)
        }
    }

    fn visit_value<T>(&mut self) -> Result<T, Error> where T: serde::Deserialize {
        if self.state == 0 {
            self.state += 1;
            let mut de = serde::de::value::ValueDeserializer::<Error>::into_deserializer(self.ty);
            Ok(try!(T::deserialize(&mut de)))
        } else if self.state == 1 {
            self.state += 1;
            let mut de = serde::de::value::ValueDeserializer::<Error>::into_deserializer(
                serde::bytes::Bytes::from(self.data.as_slice()));
            Ok(try!(T::deserialize(&mut de)))
        } else {
            Err(serde::de::Error::end_of_stream())
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        if self.state > 1 {
            Ok(())
        } else {
            Err(serde::de::Error::invalid_length(2 - self.state as usize))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (2 - self.state as usize, Some(2 - self.state as usize))
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

impl<F: FnMut(&mut [u8]) -> Result<(), Error>> serde::Deserializer for Deserializer<F> {
    type Error = Error;

    fn deserialize<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        let mut buf = [0];
        try!(self.input(&mut buf));
        self.parse_as(visitor, buf[0])
    }

    fn deserialize_bool<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_u64<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_usize<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u8<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i64<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_isize<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i8<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_i64(visitor)
    }

    fn deserialize_f64<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_f32<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_f64(visitor)
    }

    fn deserialize_str<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_char<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_str(visitor)
    }

    fn deserialize_string<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_str(visitor)
    }

    fn deserialize_unit<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_option<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_seq<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_seq_fixed_size<V>(&mut self, _: usize, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_seq(visitor)
    }

    fn deserialize_bytes<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_map<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_unit_struct<V>(&mut self, _: &'static str, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(&mut self, _: &'static str, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_tuple_struct<V>(&mut self, _: &'static str, len: usize, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_struct<V>(&mut self, _: &'static str, _: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_map(visitor)
    }

    fn deserialize_struct_field<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
    }

    fn deserialize_tuple<V>(&mut self, len: usize, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize_seq_fixed_size(len, visitor)
    }

    fn deserialize_enum<V>(&mut self, _: &'static str, _: &'static [&'static str], _: V) -> Result<V::Value, Error>
        where V: serde::de::EnumVisitor {
        Err(serde::Error::invalid_type(serde::de::Type::Enum))
    }

    fn deserialize_ignored_any<V>(&mut self, visitor: V) -> Result<V::Value, Error>
        where V: serde::de::Visitor {
        self.deserialize(visitor)
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

    fn parse_as<V>(&mut self, mut visitor: V, ty: u8) -> Result<V::Value, Error> 
        where V: serde::de::Visitor {
        match ty {
            v if POS_FIXINT.contains(v) => {
                visitor.visit_u8(v)
            }
            v if NEG_FIXINT.contains(v) => {
                visitor.visit_i8(unsafe {mem::transmute(v)})
            }
            v if FIXMAP.contains(v) => {
                let size = (v & !FIXMAP_MASK) as usize * 2;
                visitor.visit_map(SeqVisitor::new(self, size))
            }
            v if FIXARRAY.contains(v) => {
                let size = (v & !FIXARRAY_MASK) as usize;
                visitor.visit_seq(SeqVisitor::new(self, size))
            }
            v if FIXSTR.contains(v) => {
                let mut buf = vec![0; (v & !FIXSTR_MASK) as usize];
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
            EXT8 => {
                let mut buf = [0];
                try!(self.input(&mut buf));
                let size = buf[0] as usize;
                try!(self.input(&mut buf));
                let ty: i8 = unsafe {mem::transmute(buf[0])};
                let mut buf = vec![0; size];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor {
                    state: 0,
                    ty: ty,
                    data: buf
                })
            }
            EXT16 => {
                let mut buf = [0; U16_BYTES];
                try!(self.input(&mut buf));
                let size = BigEndian::read_u16(&buf) as usize;
                try!(self.input(&mut buf));
                let ty: i8 = unsafe {mem::transmute(buf[0])};
                let mut buf = vec![0; size];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor {
                    state: 0,
                    ty: ty,
                    data: buf
                })
            }
            EXT32 => {
                let mut buf = [0; U32_BYTES];
                try!(self.input(&mut buf));
                let size = BigEndian::read_u32(&buf) as usize;
                try!(self.input(&mut buf));
                let ty: i8 = unsafe {mem::transmute(buf[0])};
                let mut buf = vec![0; size];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor {
                    state: 0,
                    ty: ty,
                    data: buf
                })
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
            FIXEXT1 => {
                let mut buf = [0];
                try!(self.input(&mut buf));
                let ty: i8 = unsafe {mem::transmute(buf[0])};
                let mut buf = vec![0];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor {
                    state: 0,
                    ty: ty,
                    data: buf
                })
            }
            FIXEXT2 => {
                let mut buf = [0];
                try!(self.input(&mut buf));
                let ty: i8 = unsafe {mem::transmute(buf[0])};
                let mut buf = vec![0; 2];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor {
                    state: 0,
                    ty: ty,
                    data: buf
                })
            }
            FIXEXT4 => {
                let mut buf = [0];
                try!(self.input(&mut buf));
                let ty: i8 = unsafe {mem::transmute(buf[0])};
                let mut buf = vec![0; 4];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor {
                    state: 0,
                    ty: ty,
                    data: buf
                })
            }
            FIXEXT8 => {
                let mut buf = [0];
                try!(self.input(&mut buf));
                let ty: i8 = unsafe {mem::transmute(buf[0])};
                let mut buf = vec![0; 8];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor {
                    state: 0,
                    ty: ty,
                    data: buf
                })
            }
            FIXEXT16 => {
                let mut buf = [0];
                try!(self.input(&mut buf));
                let ty: i8 = unsafe {mem::transmute(buf[0])};
                let mut buf = vec![0; 16];
                try!(self.input(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor {
                    state: 0,
                    ty: ty,
                    data: buf
                })
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

#[cfg(test)]
mod test {
    use collections::{String, Vec};
    use collections::btree_map::BTreeMap;

    #[test]
    fn positive_fixint_test() {
        let value: u8 = ::from_bytes(&[0x17]).unwrap();
        assert_eq!(value, 23);
    }

    #[test]
    fn negative_fixint_test() {
        let value: i8 = ::from_bytes(&[0xfb]).unwrap();
        assert_eq!(value, -5);
    }

    #[test]
    fn uint8_test() {
        let value: u8 = ::from_bytes(&[0xcc, 0x9a]).unwrap();
        assert_eq!(value, 154);
    }

    #[test]
    fn fixstr_test() {
        let value: String = ::from_bytes(&[0xac, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20,
                                           0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21]).unwrap();
        assert_eq!(value, "Hello World!");
    }

    #[test]
    fn str8_test() {
        let s: &str = "The quick brown fox jumps over the lazy dog";
        let mut fixture: Vec<u8> = vec![];
        fixture.push(0xd9);
        fixture.push(s.len() as u8);
        fixture.extend_from_slice(s.as_bytes());
        let value: String = ::from_bytes(fixture.as_slice()).unwrap();
        assert_eq!(value, s);
    }

    #[test]
    fn fixarr_test() {
        let v: Vec<u8> = ::from_bytes(&[0x94, 0x05, 0x08, 0x14, 0xcc, 0xe7]).unwrap();
        assert_eq!(v, &[5, 8, 20, 231]);
    }

    #[test]
    fn array16_test() {
        let v: Vec<isize> = ::from_bytes(&[0xdc,
                                           0x00, 0x11,
                                           0xfb,  0x10,  0x65,  0xd0, 0xd3,  0xcc, 0xb8,
                                           0x59,  0x3e,  0xd1, 0xff, 0x17,  0xd0, 0xdf,  0xd1, 0x01, 0x30,
                                           0x4c, 0x5a, 0x17, 0x6c, 0x2d,
                                           0xfd, 0x02]).unwrap();

        assert_eq!(v, &[-5, 16, 101, -45, 184,
                       89, 62, -233, -33, 304,
                       76, 90, 23, 108, 45,
                       -3, 2]);
    }

    #[test]
    fn fixmap_test() {
        let mut map: BTreeMap<String, usize> = ::from_bytes(&[0x83,
                                                              0xa3, 0x6f, 0x6e, 0x65,  0x01,
                                                              0xa5, 0x74, 0x68, 0x72, 0x65, 0x65,  0x03,
                                                              0xa3, 0x74, 0x77, 0x6f,  0x02]).unwrap();
        assert_eq!(map.remove(&format!("one")), Some(1));
        assert_eq!(map.remove(&format!("two")), Some(2));
        assert_eq!(map.remove(&format!("three")), Some(3));
        assert!(map.is_empty());
    }
}

