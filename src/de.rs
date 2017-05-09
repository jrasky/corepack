//! The main deserializer mux.
//
// This Source Code Form is subject to the terms of the Mozilla Public License,
// v. 2.0. If a copy of the MPL was not distributed with this file, You can
// obtain one at https://mozilla.org/MPL/2.0/.
use collections::String;

use byteorder::{ByteOrder, BigEndian, LittleEndian};
use serde::de::Error;

use serde;

use seq_visitor::*;
use ext_visitor::*;
use variant_visitor::*;

use defs::*;

/// The corepack Deserializer struct. Contains a closure that should copy
/// the next bytes availabel into the given byte buffer.
pub struct Deserializer<F: FnMut(&mut [u8]) -> Result<()>> {
    input: F,
    peek_ty: Option<u8>
}

impl<F: FnMut(&mut [u8]) -> Result<()>> Deserializer<F> {
    /// Create a new Deserializer given an input function.
    pub const fn new(input: F) -> Deserializer<F> {
        Deserializer { input: input, peek_ty: None }
    }

    fn parse_as<'a, V>(&mut self, visitor: V, ty: u8) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        match ty {
            v if POS_FIXINT.contains(v) => visitor.visit_u8(v),
            v if NEG_FIXINT.contains(v) => visitor.visit_i8(LittleEndian::read_i16(&[v, 0]) as i8),
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
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_string(try!(String::from_utf8(buf)
                    .map_err(|e| Error::custom(format!("UTF8 Error: {}", e)))))
            }
            NIL => visitor.visit_unit(),
            FALSE => visitor.visit_bool(false),
            TRUE => visitor.visit_bool(true),
            BIN8 => {
                let mut buf = [0];
                try!((self.input)(&mut buf));
                let mut buf = vec![0; buf[0] as usize];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_byte_buf(buf)
            }
            BIN16 => {
                let mut buf = [0; U16_BYTES];
                try!((self.input)(&mut buf));
                let mut buf = vec![0; BigEndian::read_u16(&buf) as usize];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_byte_buf(buf)
            }
            BIN32 => {
                let mut buf = [0; U32_BYTES];
                try!((self.input)(&mut buf));
                let mut buf = vec![0; BigEndian::read_u32(&buf)as usize];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_byte_buf(buf)
            }
            EXT8 => {
                let mut buf = [0];
                try!((self.input)(&mut buf));
                let size = buf[0] as usize;
                try!((self.input)(&mut buf));
                let ty: i8 = LittleEndian::read_i16(&[buf[0], 0]) as i8;
                let mut buf = vec![0; size];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor::new(ty, buf))
            }
            EXT16 => {
                let mut buf = [0; U16_BYTES];
                try!((self.input)(&mut buf));
                let size = BigEndian::read_u16(&buf) as usize;
                try!((self.input)(&mut buf[..1]));
                let ty: i8 = LittleEndian::read_i16(&[buf[0], 0]) as i8;
                let mut buf = vec![0; size];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor::new(ty, buf))
            }
            EXT32 => {
                let mut buf = [0; U32_BYTES];
                try!((self.input)(&mut buf));
                let size = BigEndian::read_u32(&buf) as usize;
                try!((self.input)(&mut buf[..1]));
                let ty: i8 = LittleEndian::read_i16(&[buf[0], 0]) as i8;
                let mut buf = vec![0; size];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor::new(ty, buf))
            }
            UINT8 => {
                let mut buf = [0];
                try!((self.input)(&mut buf));
                visitor.visit_u8(buf[0])
            }
            UINT16 => {
                let mut buf = [0; U16_BYTES];
                try!((self.input)(&mut buf));
                visitor.visit_u16(BigEndian::read_u16(&buf))
            }
            UINT32 => {
                let mut buf = [0; U32_BYTES];
                try!((self.input)(&mut buf));
                visitor.visit_u32(BigEndian::read_u32(&buf))
            }
            UINT64 => {
                let mut buf = [0; U64_BYTES];
                try!((self.input)(&mut buf));
                visitor.visit_u64(BigEndian::read_u64(&buf))
            }
            INT8 => {
                let mut buf = [0];
                try!((self.input)(&mut buf));
                visitor.visit_i8(LittleEndian::read_i16(&[buf[0], 0]) as i8)
            }
            INT16 => {
                let mut buf = [0; U16_BYTES];
                try!((self.input)(&mut buf));
                visitor.visit_i16(BigEndian::read_i16(&buf))
            }
            INT32 => {
                let mut buf = [0; U32_BYTES];
                try!((self.input)(&mut buf));
                visitor.visit_i32(BigEndian::read_i32(&buf))
            }
            INT64 => {
                let mut buf = [0; U64_BYTES];
                try!((self.input)(&mut buf));
                visitor.visit_i64(BigEndian::read_i64(&buf))
            }
            FIXEXT1 => {
                let mut buf = [0];
                try!((self.input)(&mut buf));
                let ty: i8 = LittleEndian::read_i16(&[buf[0], 0]) as i8;
                let mut buf = vec![0];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor::new(ty, buf))
            }
            FIXEXT2 => {
                let mut buf = [0];
                try!((self.input)(&mut buf));
                let ty: i8 = LittleEndian::read_i16(&[buf[0], 0]) as i8;
                let mut buf = vec![0; 2];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor::new(ty, buf))
            }
            FIXEXT4 => {
                let mut buf = [0];
                try!((self.input)(&mut buf));
                let ty: i8 = LittleEndian::read_i16(&[buf[0], 0]) as i8;
                let mut buf = vec![0; 4];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor::new(ty, buf))
            }
            FIXEXT8 => {
                let mut buf = [0];
                try!((self.input)(&mut buf));
                let ty: i8 = LittleEndian::read_i16(&[buf[0], 0]) as i8;
                let mut buf = vec![0; 8];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor::new(ty, buf))
            }
            FIXEXT16 => {
                let mut buf = [0];
                try!((self.input)(&mut buf));
                let ty: i8 = LittleEndian::read_i16(&[buf[0], 0]) as i8;
                let mut buf = vec![0; 16];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_map(ExtVisitor::new(ty, buf))
            }
            STR8 => {
                let mut buf = [0];
                try!((self.input)(&mut buf));
                let mut buf = vec![0; buf[0] as usize];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_string(try!(String::from_utf8(buf)
                    .map_err(|e| Error::custom(format!("UTF8 Error: {}", e)))))
            }
            STR16 => {
                let mut buf = [0; U16_BYTES];
                try!((self.input)(&mut buf));
                let mut buf = vec![0; BigEndian::read_u16(&buf) as usize];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_string(try!(String::from_utf8(buf)
                    .map_err(|e| Error::custom(format!("UTF8 Error: {}", e)))))
            }
            STR32 => {
                let mut buf = [0; U32_BYTES];
                try!((self.input)(&mut buf));
                let mut buf = vec![0; BigEndian::read_u32(&buf) as usize];
                try!((self.input)(buf.as_mut_slice()));
                visitor.visit_string(try!(String::from_utf8(buf)
                    .map_err(|e| Error::custom(format!("UTF8 Error: {}", e)))))
            }
            ARRAY16 => {
                let mut buf = [0; U16_BYTES];
                try!((self.input)(&mut buf));
                let size = BigEndian::read_u16(&buf);
                visitor.visit_seq(SeqVisitor::new(self, size as usize))
            }
            ARRAY32 => {
                let mut buf = [0; U32_BYTES];
                try!((self.input)(&mut buf));
                let size = BigEndian::read_u32(&buf);
                visitor.visit_seq(SeqVisitor::new(self, size as usize))
            }
            MAP16 => {
                let mut buf = [0; U16_BYTES];
                try!((self.input)(&mut buf));
                let size = BigEndian::read_u16(&buf);
                visitor.visit_map(SeqVisitor::new(self, size as usize * 2))
            }
            MAP32 => {
                let mut buf = [0; U16_BYTES];
                try!((self.input)(&mut buf));
                let size = BigEndian::read_u32(&buf);
                visitor.visit_map(SeqVisitor::new(self, size as usize * 2))
            }
            _ => Err(Error::custom("Bad type")),
        }
    }
}

impl<'a, 'b, F: FnMut(&mut [u8]) -> Result<()>> serde::Deserializer<'a> for &'b mut Deserializer<F> {
    type Error = serde::de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        if let Some(ty) = self.peek_ty.take() {
            self.parse_as(visitor, ty)
        } else {
            let mut buf = [0];
            try!((self.input)(&mut buf));
            self.parse_as(visitor, buf[0])
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        let mut buf = [0];
        try!((self.input)(&mut buf));
        let ty = buf[0];

        if ty == NIL {
            visitor.visit_none()
        } else {
            self.peek_ty = Some(ty);
            visitor.visit_some(self)
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_unit_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_tuple_struct<V>(self,
                                   _: &'static str,
                                   len: usize,
                                   visitor: V)
                                   -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_struct<V>(self,
                             _: &'static str,
                             _: &'static [&'static str],
                             visitor: V)
                             -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_enum<V>(self,
                           _: &'static str,
                           variants: &'static [&'static str],
                           visitor: V)
                           -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        visitor.visit_enum(VariantVisitor::new(self, variants))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value> 
        where V: serde::de::Visitor<'a>
    {
        self.deserialize_any(visitor)
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
        let value: String = ::from_bytes(&[0xac, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f,
                                           0x72, 0x6c, 0x64, 0x21])
            .unwrap();
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
        let v: Vec<isize> = ::from_bytes(&[0xdc, 0x00, 0x11, 0xfb, 0x10, 0x65, 0xd0, 0xd3, 0xcc,
                                           0xb8, 0x59, 0x3e, 0xd1, 0xff, 0x17, 0xd0, 0xdf, 0xd1,
                                           0x01, 0x30, 0x4c, 0x5a, 0x17, 0x6c, 0x2d, 0xfd, 0x02])
            .unwrap();

        assert_eq!(v,
                   &[-5, 16, 101, -45, 184, 89, 62, -233, -33, 304, 76, 90, 23, 108, 45, -3, 2]);
    }

    #[test]
    fn fixmap_test() {
        let mut map: BTreeMap<String, usize> = ::from_bytes(&[0x83, 0xa3, 0x6f, 0x6e, 0x65, 0x01,
                                                              0xa5, 0x74, 0x68, 0x72, 0x65, 0x65,
                                                              0x03, 0xa3, 0x74, 0x77, 0x6f, 0x02])
            .unwrap();
        assert_eq!(map.remove(&format!("one")), Some(1));
        assert_eq!(map.remove(&format!("two")), Some(2));
        assert_eq!(map.remove(&format!("three")), Some(3));
        assert!(map.is_empty());
    }
}
