use std::result;

use collections::{Vec, String};

use byteorder::{ByteOrder, BigEndian, LittleEndian};

use serde::ser::{SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeMap,
                 SerializeTupleStruct};

use serde;

use defs::*;
use error::*;
use seq_serializer::*;
use map_serializer::*;
use map_variant_serializer::*;

pub struct Serializer<F: FnMut(&[u8]) -> Result<()>> {
    output: F,
}

impl<F: FnMut(&[u8]) -> Result<()>> Serializer<F> {
    pub const fn new(output: F) -> Serializer<F> {
        Serializer { output: output }
    }

    fn serialize_signed(&mut self, value: i64) -> Result<()> {
        if value >= FIXINT_MIN as i64 && value <= FIXINT_MAX as i64 {
            let mut buf = [0; U16_BYTES];
            LittleEndian::write_i16(&mut buf, value as i16);
            (self.output)(&buf[..1])
        } else if value >= i8::min_value() as i64 && value <= i8::max_value() as i64 {
            let mut buf = [0; U16_BYTES];
            LittleEndian::write_i16(&mut buf, value as i16);
            (self.output)(&[INT8, buf[0]])
        } else if value >= 0 && value <= u8::max_value() as i64 {
            let mut buf = [0; U16_BYTES];
            LittleEndian::write_i16(&mut buf, value as i16);
            (self.output)(&[UINT8, buf[0]])
        } else if value >= i16::min_value() as i64 && value <= i16::max_value() as i64 {
            let mut buf = [INT16; U16_BYTES + 1];
            BigEndian::write_i16(&mut buf[1..], value as i16);
            (self.output)(&buf)
        } else if value >= 0 && value <= u16::max_value() as i64 {
            let mut buf = [UINT16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], value as u16);
            (self.output)(&buf)
        } else if value >= i32::min_value() as i64 && value <= i32::max_value() as i64 {
            let mut buf = [INT32; U32_BYTES + 1];
            BigEndian::write_i32(&mut buf[1..], value as i32);
            (self.output)(&buf)
        } else if value >= 0 && value <= u32::max_value() as i64 {
            let mut buf = [UINT32; U16_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], value as u32);
            (self.output)(&buf)
        } else {
            let mut buf = [INT64; U64_BYTES + 1];
            BigEndian::write_i64(&mut buf[1..], value);
            (self.output)(&buf)
        }
    }

    fn serialize_unsigned(&mut self, value: u64) -> Result<()> {
        if value <= FIXINT_MAX as u64 {
            (self.output)(&[value as u8])
        } else if value <= u8::max_value() as u64 {
            (self.output)(&[UINT8, value as u8])
        } else if value <= u16::max_value() as u64 {
            let mut buf = [UINT16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], value as u16);
            (self.output)(&buf)
        } else if value <= u32::max_value() as u64 {
            let mut buf = [UINT32; U32_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], value as u32);
            (self.output)(&buf)
        } else {
            let mut buf = [UINT64; U64_BYTES + 1];
            BigEndian::write_u64(&mut buf[1..], value);
            (self.output)(&buf)
        }
    }

    fn serialize_bool(&mut self, value: bool) -> Result<()> {
        if value {
            (self.output)(&[TRUE])
        } else {
            (self.output)(&[FALSE])
        }
    }

    fn serialize_f32(&mut self, value: f32) -> Result<()> {
        let mut buf = [FLOAT32; U32_BYTES + 1];
        BigEndian::write_f32(&mut buf[1..], value);
        (self.output)(&buf)
    }

    fn serialize_f64(&mut self, value: f64) -> Result<()> {
        let mut buf = [FLOAT64; U64_BYTES + 1];
        BigEndian::write_f64(&mut buf[1..], value);
        (self.output)(&buf)
    }

    fn serialize_bytes(&mut self, value: &[u8]) -> Result<()> {
        if value.len() <= MAX_BIN8 {
            try!((self.output)(&[BIN8, value.len() as u8]));
        } else if value.len() <= MAX_BIN16 {
            let mut buf = [BIN16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], value.len() as u16);
            try!((self.output)(&buf));
        } else if value.len() <= MAX_BIN32 {
            let mut buf = [BIN32; U32_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], value.len() as u32);
            try!((self.output)(&buf));
        } else {
            return Err(Error::simple(Reason::TooBig));
        }

        (self.output)(value)
    }

    fn serialize_str(&mut self, value: &str) -> Result<()> {
        if value.len() <= MAX_FIXSTR {
            try!((self.output)(&[value.len() as u8 | FIXSTR_MASK]));
        } else if value.len() <= MAX_STR8 {
            try!((self.output)(&[STR8, value.len() as u8]));
        } else if value.len() <= MAX_STR16 {
            let mut buf = [STR16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], value.len() as u16);
            try!((self.output)(&buf));
        } else if value.len() <= MAX_STR32 {
            let mut buf = [STR32; U32_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], value.len() as u32);
            try!((self.output)(&buf));
        } else {
            return Err(Error::simple(Reason::TooBig));
        }

        (self.output)(value.as_bytes())
    }

    fn serialize_unit(&mut self) -> Result<()> {
        (self.output)(&[NIL])
    }
}

impl<'a, F: 'a + FnMut(&[u8]) -> Result<()>> serde::Serializer for &'a mut Serializer<F> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqSerializer<'a, F>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeTupleStruct = Self::SerializeTuple;
    type SerializeTupleVariant = Self::SerializeTuple;

    type SerializeMap = MapSerializer<'a, F>;
    type SerializeStruct = Self::SerializeMap;
    type SerializeStructVariant = MapVariantSerializer<'a, F>;

    fn serialize_bool(self, v: bool) -> Result<()> {
        Serializer::serialize_bool(self, v)
    }

    fn serialize_i64(self, value: i64) -> Result<()> {
        Serializer::serialize_signed(self, value)
    }

    fn serialize_u64(self, value: u64) -> Result<()> {
        Serializer::serialize_unsigned(self, value)
    }

    fn serialize_f32(self, value: f32) -> Result<()> {
        Serializer::serialize_f32(self, value)
    }

    fn serialize_f64(self, value: f64) -> Result<()> {
        Serializer::serialize_f64(self, value)
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        Serializer::serialize_bytes(self, value)
    }

    fn serialize_str(self, value: &str) -> Result<()> {
        Serializer::serialize_str(self, value)
    }

    fn serialize_unit(self) -> Result<()> {
        Serializer::serialize_unit(self)
    }

    fn serialize_i8(self, value: i8) -> Result<()> {
        Serializer::serialize_signed(self, value as i64)
    }

    fn serialize_i16(self, value: i16) -> Result<()> {
        Serializer::serialize_signed(self, value as i64)
    }

    fn serialize_i32(self, value: i32) -> Result<()> {
        Serializer::serialize_signed(self, value as i64)
    }

    fn serialize_u8(self, value: u8) -> Result<()> {
        Serializer::serialize_unsigned(self, value as u64)
    }

    fn serialize_u16(self, value: u16) -> Result<()> {
        Serializer::serialize_unsigned(self, value as u64)
    }

    fn serialize_u32(self, value: u32) -> Result<()> {
        Serializer::serialize_unsigned(self, value as u64)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        let mut string = String::new();
        string.push(v);

        self.serialize_str(&*string)
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _: &'static str, index: usize, _: &'static str) -> Result<()> {
        Serializer::serialize_unsigned(self, index as u64)
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
        where T: ?Sized + serde::Serialize
    {
        let mut seq = self.serialize_tuple_struct(name, 1)?;
        SerializeTupleStruct::serialize_field(&mut seq, &value)?;
        SerializeTupleStruct::end(seq)
    }

    fn serialize_newtype_variant<T>(self,
                                    name: &'static str,
                                    variant_index: usize,
                                    variant: &'static str,
                                    value: &T)
                                    -> Result<()>
        where T: ?Sized + serde::Serialize
    {
        let mut seq = self.serialize_tuple_variant(name, variant_index, variant, 1)?;
        SerializeTupleStruct::serialize_field(&mut seq, &value)?;
        SerializeTupleStruct::end(seq)
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<V>(self, value: &V) -> Result<()>
        where V: ?Sized + serde::Serialize
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> result::Result<Self::SerializeSeq, Error> {
        Ok(SeqSerializer::new(&mut self.output))
    }

    fn serialize_seq_fixed_size(self, size: usize) -> result::Result<Self::SerializeSeq, Error> {
        self.serialize_seq(Some(size))
    }

    fn serialize_tuple(self, len: usize) -> result::Result<Self::SerializeTuple, Error> {
        self.serialize_seq_fixed_size(len)
    }

    fn serialize_tuple_struct(self,
                              _: &'static str,
                              len: usize)
                              -> result::Result<Self::SerializeTupleStruct, Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(self,
                               _: &'static str,
                               index: usize,
                               _: &'static str,
                               len: usize)
                               -> result::Result<Self::SerializeTupleVariant, Error> {
        let mut seq = self.serialize_tuple(len + 1)?;
        // serialize the variant index as an extra element at the front
        seq.serialize_element(&index)?;

        Ok(seq)
    }

    fn serialize_map(self, len: Option<usize>) -> result::Result<Self::SerializeMap, Error> {
        Ok(MapSerializer::new(&mut self.output))
    }

    fn serialize_struct(self,
                        _: &'static str,
                        len: usize)
                        -> result::Result<Self::SerializeStruct, Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(self,
                                name: &'static str,
                                index: usize,
                                _: &'static str,
                                _: usize)
                                -> result::Result<Self::SerializeStructVariant, Error> {
        Ok(MapVariantSerializer::new(index, &mut self.output))
    }
}

#[cfg(test)]
mod test {
    use collections::{Vec, String};
    use collections::btree_map::BTreeMap;

    #[test]
    fn positive_fixint_test() {
        let v: u8 = 23;
        assert_eq!(::to_bytes(v).unwrap(), &[0x17]);
    }
    #[test]
    fn negative_fixint_test() {
        let v: i8 = -5;
        assert_eq!(::to_bytes(v).unwrap(), &[0xfb]);
    }

    #[test]
    fn uint8_test() {
        let v: u8 = 154;
        assert_eq!(::to_bytes(v).unwrap(), &[0xcc, 0x9a]);
    }

    #[test]
    fn fixstr_test() {
        let s: &str = "Hello World!";
        assert_eq!(::to_bytes(s).unwrap(),
                   &[0xac, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21]);
    }

    #[test]
    fn str8_test() {
        let s: &str = "The quick brown fox jumps over the lazy dog";
        let mut fixture: Vec<u8> = vec![];
        fixture.push(0xd9);
        fixture.push(s.len() as u8);
        fixture.extend_from_slice(s.as_bytes());
        assert_eq!(::to_bytes(s).unwrap(), fixture);
    }

    #[test]
    fn fixarr_test() {
        let v: Vec<u8> = vec![5, 8, 20, 231];
        assert_eq!(::to_bytes(v).unwrap(),
                   &[0x94, 0x05, 0x08, 0x14, 0xcc, 0xe7]);
    }

    #[test]
    fn array16_test() {
        let v: Vec<isize> = vec![-5, 16, 101, -45, 184, 89, 62, -233, -33, 304, 76, 90, 23, 108,
                                 45, -3, 2];
        assert_eq!(::to_bytes(v).unwrap(),
                   &[0xdc, 0x00, 0x11, 0xfb, 0x10, 0x65, 0xd0, 0xd3, 0xcc, 0xb8, 0x59, 0x3e,
                     0xd1, 0xff, 0x17, 0xd0, 0xdf, 0xd1, 0x01, 0x30, 0x4c, 0x5a, 0x17, 0x6c,
                     0x2d, 0xfd, 0x02]);
    }

    #[test]
    fn fixmap_test() {
        let mut map: BTreeMap<String, usize> = BTreeMap::new();
        map.insert("one".into(), 1);
        map.insert("two".into(), 2);
        map.insert("three".into(), 3);
        assert_eq!(::to_bytes(map).unwrap(),
                   &[0x83, 0xa3, 0x6f, 0x6e, 0x65, 0x01, 0xa5, 0x74, 0x68, 0x72, 0x65, 0x65,
                     0x03, 0xa3, 0x74, 0x77, 0x6f, 0x02]);
    }
}
