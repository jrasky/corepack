use std::result;

use collections::{Vec, String};

use byteorder::{ByteOrder, BigEndian, LittleEndian};

use serde::ser::{SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeMap,
                 SerializeTupleStruct};

use serde;

use defs::*;
use error::*;

pub type Result = result::Result<(), Error>;

pub struct Serializer<F: FnMut(&[u8]) -> Result> {
    output: F,
}

pub struct MapVariantSerializer<F: FnMut(&[u8]) -> Result> {
    map_container: SeqSerializer<F>,
    variant: usize,
}

pub struct SeqSerializer<F: FnMut(&[u8]) -> Result> {
    size: usize,
    buffer: Vec<u8>,
    output: F,
}

impl<F: FnMut(&[u8]) -> Result> MapVariantSerializer<F> {
    fn new(variant: usize, output: F) -> MapVariantSerializer<F> {
        let mut map_container = SeqSerializer::new(output);

        MapVariantSerializer {
            map_container: map_container,
            variant: variant,
        }
    }
}

impl<F: FnMut(&[u8]) -> Result> serde::ser::SerializeStructVariant for MapVariantSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result
        where T: ?Sized + serde::Serialize
    {
        serde::ser::SerializeMap::serialize_entry(&mut self.map_container, key, value)
    }

    fn end(mut self) -> Result {
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
            serde::ser::SerializeTuple::serialize_element(&mut variant_container, &self.variant);

            variant_container.size = 2;

            // write out the variant container
            serde::ser::SerializeSeq::end(variant_container)?;
        }

        // output the buffer
        (self.map_container.output)(buffer.as_slice())?;

        // then write out the map
        serde::ser::SerializeMap::end(self.map_container)
    }
}

impl<F: FnMut(&[u8]) -> Result> SeqSerializer<F> {
    fn new(output: F) -> SeqSerializer<F> {
        SeqSerializer {
            size: 0,
            buffer: vec![],
            output: output,
        }
    }
}

impl<F: FnMut(&[u8]) -> Result> serde::ser::SerializeMap for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result
        where T: ?Sized + serde::Serialize
    {
        serde::ser::SerializeSeq::serialize_element(self, key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result
        where T: ?Sized + serde::Serialize
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(mut self) -> Result {
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

impl<F: FnMut(&[u8]) -> Result> serde::ser::SerializeStruct for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result
        where T: ?Sized + serde::Serialize
    {
        serde::ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<F: FnMut(&[u8]) -> Result> serde::ser::SerializeTupleVariant for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result
        where T: ?Sized + serde::Serialize
    {
        serde::ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result {
        serde::ser::SerializeStruct::end(self)
    }
}

impl<F: FnMut(&[u8]) -> Result> serde::ser::SerializeTupleStruct for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result
        where T: ?Sized + serde::Serialize
    {
        serde::ser::SerializeTuple::serialize_element(self, value)
    }

    fn end(self) -> Result {
        serde::ser::SerializeTuple::end(self)
    }
}

impl<F: FnMut(&[u8]) -> Result> serde::ser::SerializeTuple for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result
        where T: ?Sized + serde::Serialize
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result {
        serde::ser::SerializeSeq::end(self)
    }
}

impl<F: FnMut(&[u8]) -> Result> serde::ser::SerializeSeq for SeqSerializer<F> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result
        where T: ?Sized + serde::Serialize
    {
        self.size += 1;

        let target = Serializer::new(|bytes| {
            self.buffer.extend_from_slice(bytes);
            Ok(())
        });

        value.serialize(target)
    }

    fn end(mut self) -> Result {
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

impl<F: FnMut(&[u8]) -> Result> Serializer<F> {
    pub const fn new(output: F) -> Serializer<F> {
        Serializer { output: output }
    }

    fn output(&mut self, buf: &[u8]) -> Result {
        self.output.call_mut((buf,))
    }
}

impl<F: FnMut(&[u8]) -> Result> serde::Serializer for Serializer<F> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SeqSerializer<F>;
    type SerializeTuple = Self::SerializeSeq;
    type SerializeTupleStruct = Self::SerializeTuple;
    type SerializeTupleVariant = Self::SerializeTuple;

    type SerializeMap = SeqSerializer<F>;
    type SerializeStruct = Self::SerializeMap;
    type SerializeStructVariant = MapVariantSerializer<F>;

    fn serialize_bool(mut self, v: bool) -> Result {
        if v {
            (self.output)(&[TRUE])
        } else {
            (self.output)(&[FALSE])
        }
    }

    fn serialize_i64(mut self, value: i64) -> Result {
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

    fn serialize_i8(self, value: i8) -> Result {
        self.serialize_i64(value as i64)
    }

    fn serialize_i16(self, value: i16) -> Result {
        self.serialize_i64(value as i64)
    }

    fn serialize_i32(self, value: i32) -> Result {
        self.serialize_i64(value as i64)
    }

    fn serialize_u64(mut self, value: u64) -> Result {
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

    fn serialize_u8(self, value: u8) -> Result {
        self.serialize_u64(value as u64)
    }

    fn serialize_u16(self, value: u16) -> Result {
        self.serialize_u64(value as u64)
    }

    fn serialize_u32(self, value: u32) -> Result {
        self.serialize_u64(value as u64)
    }

    fn serialize_f32(mut self, value: f32) -> Result {
        let mut buf = [FLOAT32; U32_BYTES + 1];
        BigEndian::write_f32(&mut buf[1..], value);
        (self.output)(&buf)
    }

    fn serialize_f64(mut self, value: f64) -> Result {
        let mut buf = [FLOAT64; U64_BYTES + 1];
        BigEndian::write_f64(&mut buf[1..], value);
        (self.output)(&buf)
    }

    fn serialize_str(mut self, value: &str) -> Result {
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

    fn serialize_char(self, v: char) -> Result {
        let mut string = String::new();
        string.push(v);

        self.serialize_str(&*string)
    }

    fn serialize_unit(mut self) -> Result {
        (self.output)(&[NIL])
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result {
        self.serialize_unit()
    }

    fn serialize_unit_variant(self, _: &'static str, index: usize, _: &'static str) -> Result {
        self.serialize_u64(index as u64)
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result
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
                                    -> Result
        where T: ?Sized + serde::Serialize
    {
        let mut seq = self.serialize_tuple_variant(name, variant_index, variant, 1)?;
        SerializeTupleStruct::serialize_field(&mut seq, &value)?;
        SerializeTupleStruct::end(seq)
    }

    fn serialize_none(self) -> Result {
        self.serialize_unit()
    }

    fn serialize_some<V>(self, value: &V) -> Result
        where V: ?Sized + serde::Serialize
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> result::Result<Self::SerializeSeq, Error> {
        Ok(SeqSerializer::new(self.output))
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
        Ok(SeqSerializer::new(self.output))
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
        Ok(MapVariantSerializer::new(index, self.output))
    }

    fn serialize_bytes(mut self, value: &[u8]) -> Result {
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
