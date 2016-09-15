use std::mem;
use std::result;

use collections::{Vec, String};

use byteorder::{ByteOrder, BigEndian};

use serde::Serialize;

use serde;

use defs::*;
use error::*;

pub type Result = result::Result<(), Error>;

pub struct Serializer<F: FnMut(&[u8]) -> Result> {
    output: F
}

pub struct MapState {
    keys: Vec<::generic::Generic>,
    values: Vec<::generic::Generic>,
}

impl<F: FnMut(&[u8]) -> Result> Serializer<F> {
    pub const fn new(output: F) -> Serializer<F> {
        Serializer {
            output: output
        }
    }

    fn output(&mut self, buf: &[u8]) -> Result {
        self.output.call_mut((buf,))
    }
}

impl<F: FnMut(&[u8]) -> Result> serde::Serializer for Serializer<F> {
    type Error = Error;

    type SeqState = Vec<::generic::Generic>;
    type TupleState = Vec<::generic::Generic>;
    type TupleStructState = Vec<::generic::Generic>;
    type TupleVariantState = Vec<::generic::Generic>;

    type MapState = MapState;
    type StructState = MapState;
    type StructVariantState = MapState;

    fn serialize_bool(&mut self, v: bool) -> Result {
        if v {
            self.output(&[TRUE])
        } else {
            self.output(&[FALSE])
        }
    }

    fn serialize_i64(&mut self, value: i64) -> Result {
        if value >= FIXINT_MIN as i64 && value <= FIXINT_MAX as i64 {
            self.output(&[unsafe {mem::transmute(value as i8)}])
        } else if value >= i8::min_value() as i64 && value <= i8::max_value() as i64 {
            self.output(&[INT8, unsafe {mem::transmute(value as i8)}])
        } else if value >= 0 && value <= u8::max_value() as i64 {
            self.output(&[UINT8, unsafe {mem::transmute(value as u8)}])
        } else if value >= i16::min_value() as i64 && value <= i16::max_value() as i64 {
            let mut buf = [INT16; U16_BYTES + 1];
            BigEndian::write_i16(&mut buf[1..], value as i16);
            self.output(&buf)
        } else if value >= 0 && value <= u16::max_value() as i64 {
            let mut buf = [UINT16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], value as u16);
            self.output(&buf)
        } else if value >= i32::min_value() as i64 && value <= i32::max_value() as i64 {
            let mut buf = [INT32; U32_BYTES + 1];
            BigEndian::write_i32(&mut buf[1..], value as i32);
            self.output(&buf)
        } else if value >= 0 && value <= u32::max_value() as i64 {
            let mut buf = [UINT32; U16_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], value as u32);
            self.output(&buf)
        } else {
            let mut buf = [INT64; U64_BYTES + 1];
            BigEndian::write_i64(&mut buf[1..], value);
            self.output(&buf)
        }
    }

    fn serialize_isize(&mut self, value: isize) -> Result {
        self.serialize_i64(value as i64)
    }

    fn serialize_i8(&mut self, value: i8) -> Result {
        self.serialize_i64(value as i64)
    }

    fn serialize_i16(&mut self, value: i16) -> Result {
        self.serialize_i64(value as i64)
    }

    fn serialize_i32(&mut self, value: i32) -> Result {
        self.serialize_i64(value as i64)
    }

    fn serialize_u64(&mut self, value: u64) -> Result {
        if value <= FIXINT_MAX as u64 {
            self.output(&[value as u8])
        } else if value <= u8::max_value() as u64 {
            self.output(&[UINT8, value as u8])
        } else if value <= u16::max_value() as u64 {
            let mut buf = [UINT16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], value as u16);
            self.output(&buf)
        } else if value <= u32::max_value() as u64 {
            let mut buf = [UINT32; U32_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], value as u32);
            self.output(&buf)
        } else {
            let mut buf = [UINT64; U64_BYTES + 1];
            BigEndian::write_u64(&mut buf[1..], value);
            self.output(&buf)
        }
    }

    fn serialize_usize(&mut self, value: usize) -> Result {
        self.serialize_u64(value as u64)
    }

    fn serialize_u8(&mut self, value: u8) -> Result {
        self.serialize_u64(value as u64)
    }

    fn serialize_u16(&mut self, value: u16) -> Result {
        self.serialize_u64(value as u64)
    }

    fn serialize_u32(&mut self, value: u32) -> Result {
        self.serialize_u64(value as u64)
    }

    fn serialize_f32(&mut self, value: f32) -> Result {
        let mut buf = [FLOAT32; U32_BYTES + 1];
        BigEndian::write_f32(&mut buf[1..], value);
        self.output(&buf)
    }

    fn serialize_f64(&mut self, value: f64) -> Result {
        let mut buf = [FLOAT64; U64_BYTES + 1];
        BigEndian::write_f64(&mut buf[1..], value);
        self.output(&buf)
    }

    fn serialize_str(&mut self, value: &str) -> Result {
        if value.len() <= MAX_FIXSTR {
            try!(self.output(&[value.len() as u8 | FIXSTR_MASK]));
        } else if value.len() <= MAX_STR8 {
            try!(self.output(&[STR8, value.len() as u8]));
        } else if value.len() <= MAX_STR16 {
            let mut buf = [STR16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], value.len() as u16);
            try!(self.output(&buf));
        } else if value.len() <= MAX_STR32 {
            let mut buf = [STR32; U32_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], value.len() as u32);
            try!(self.output(&buf));
        } else {
            return Err(Error::simple(Reason::TooBig));
        }

        self.output(value.as_bytes())
    }

    fn serialize_char(&mut self, v: char) -> Result {
        let string = String::from(vec![v]);
        self.serialize_str(&*string)
    }

    fn serialize_unit(&mut self) -> Result {
        self.output(&[NIL])
    }

    fn serialize_unit_struct(&mut self, _: &'static str) -> Result {
        self.serialize_unit()
    }

    fn serialize_unit_variant(&mut self, name: &'static str, _: usize, _: &'static str) -> Result {
        self.serialize_unit_struct(name)
    }

    fn serialize_newtype_struct<T>(&mut self, name: &'static str, value: T) -> Result
        where T: serde::Serialize {
        let mut state = try!(self.serialize_tuple_struct(name, 1));
        try!(self.serialize_tuple_struct_elt(&mut state, value));
        self.serialize_tuple_struct_end(state)
    }

    fn serialize_newtype_variant<T>(&mut self, name: &'static str, variant_index: usize, variant: &'static str, value: T) -> Result
        where T: serde::Serialize {
        let mut state = try!(self.serialize_tuple_variant(name, variant_index, variant, 1));
        try!(self.serialize_tuple_variant_elt(&mut state, value));
        self.serialize_tuple_variant_end(state)
    }

    fn serialize_none(&mut self) -> Result {
        self.serialize_unit()
    }

    fn serialize_some<V>(&mut self, value: V) -> Result
        where V: serde::Serialize {
        value.serialize(self)
    }

    fn serialize_seq(&mut self, len: Option<usize>) -> result::Result<Vec<::generic::Generic>, Error> {
        if let Some(size) = len {
            Ok(Vec::with_capacity(size))
        } else {
            Ok(vec![])
        }
    }

    fn serialize_seq_fixed_size(&mut self, size: usize) -> result::Result<Vec<::generic::Generic>, Error> {
        self.serialize_seq(Some(size))
    }

    fn serialize_seq_elt<T>(&mut self, state: &mut Vec<::generic::Generic>, value: T) -> Result
        where T: serde::Serialize {
        state.push(try!(::generic::Generic::from_value(value)));

        Ok(())
    }

    fn serialize_seq_end(&mut self, state: Vec<::generic::Generic>) -> Result {
        let size = state.len();

        if size <= MAX_FIXARRAY {
            try!(self.output(&[size as u8 | FIXARRAY_MASK]));
        } else if size <= MAX_ARRAY16 {
            let mut buf = [ARRAY16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], size as u16);
            try!(self.output(&buf));
        } else if size <= MAX_ARRAY32 {
            let mut buf = [ARRAY32; U32_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], size as u32);
            try!(self.output(&buf));
        } else {
            return Err(Error::simple(Reason::TooBig));
        }

        for value in state {
            try!(value.serialize(self));
        }

        Ok(())
    }

    fn serialize_tuple(&mut self, len: usize) -> result::Result<Vec<::generic::Generic>, Error> {
        self.serialize_seq_fixed_size(len)
    }

    fn serialize_tuple_elt<T>(&mut self, state: &mut Vec<::generic::Generic>, value: T) -> Result
        where T: serde::Serialize {
        self.serialize_seq_elt(state, value)
    }

    fn serialize_tuple_end(&mut self, state: Vec<::generic::Generic>) -> Result {
        self.serialize_seq_end(state)
    }

    fn serialize_tuple_struct(&mut self, _: &'static str, len: usize) -> result::Result<Vec<::generic::Generic>, Error> {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_struct_elt<T>(&mut self, state: &mut Vec<::generic::Generic>, value: T) -> Result
        where T: serde::Serialize {
        self.serialize_tuple_elt(state, value)
    }

    fn serialize_tuple_struct_end(&mut self, state: Vec<::generic::Generic>) -> Result {
        self.serialize_tuple_end(state)
    }

    fn serialize_tuple_variant(&mut self, name: &'static str, _: usize, _: &'static str, len: usize) -> result::Result<Vec<::generic::Generic>, Error> {
        self.serialize_tuple_struct(name, len)
    }

    fn serialize_tuple_variant_elt<T>(&mut self, state: &mut Vec<::generic::Generic>, value: T) -> Result
        where T: serde::Serialize {
        self.serialize_tuple_struct_elt(state, value)
    }

    fn serialize_tuple_variant_end(&mut self, state: Vec<::generic::Generic>) -> Result {
        self.serialize_tuple_struct_end(state)
    }

    fn serialize_map(&mut self, len: Option<usize>) -> result::Result<MapState, Error> {
        if let Some(size) = len {
            Ok(MapState {
                keys: Vec::with_capacity(size),
                values: Vec::with_capacity(size),
            })
        } else {
            Ok(MapState {
                keys: vec![],
                values: vec![],
            })
        }
    }

    fn serialize_map_key<T>(&mut self, state: &mut MapState, key: T) -> Result
        where T: serde::Serialize {
        state.keys.push(try!(::generic::Generic::from_value(key)));

        Ok(())
    }

    fn serialize_map_value<T>(&mut self, state: &mut MapState, value: T) -> Result
        where T: serde::Serialize {
        state.values.push(try!(::generic::Generic::from_value(value)));

        Ok(())
    }

    fn serialize_map_end(&mut self, state: MapState) -> Result {
        if state.keys.len() != state.values.len() {
            return Err(serde::Error::custom("Map did not have same number of values as keys"));
        }

        let size = state.keys.len();

        if size <= MAX_FIXMAP {
            try!(self.output(&[size as u8 | FIXMAP_MASK]));
        } else if size <= MAX_MAP16 {
            let mut buf = [MAP16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], size as u16);
            try!(self.output(&buf));
        } else if size <= MAX_MAP32 {
            let mut buf = [MAP32; U32_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], size as u32);
            try!(self.output(&buf));
        } else {
            return Err(Error::simple(Reason::TooBig));
        }

        for (key, value) in state.keys.into_iter().zip(state.values) {
            try!(key.serialize(self));
            try!(value.serialize(self));
        }

        Ok(())
    }

    fn serialize_struct(&mut self, _: &'static str, len: usize) -> result::Result<MapState, Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_elt<V>(&mut self, state: &mut MapState, key: &'static str, value: V) -> Result
        where V: serde::Serialize {
        try!(self.serialize_map_key(state, key));
        self.serialize_map_value(state, value)
    }

    fn serialize_struct_end(&mut self, state: MapState) -> Result {
        self.serialize_map_end(state)
    }

    fn serialize_struct_variant(&mut self, name: &'static str, _: usize, _: &'static str, len: usize) -> result::Result<MapState, Error> {
        self.serialize_struct(name, len)
    }

    fn serialize_struct_variant_elt<V>(&mut self, state: &mut MapState, key: &'static str, value: V) -> Result
        where V: serde::Serialize {
        try!(self.serialize_map_key(state, key));
        self.serialize_map_value(state, value)
    }

    fn serialize_struct_variant_end(&mut self, state: MapState) -> Result {
        self.serialize_struct_end(state)
    }

    fn serialize_bytes(&mut self, value: &[u8]) -> Result {
        if value.len() <= MAX_BIN8 {
            try!(self.output(&[BIN8, value.len() as u8]));
        } else if value.len() <= MAX_BIN16 {
            let mut buf = [BIN16; U16_BYTES + 1];
            BigEndian::write_u16(&mut buf[1..], value.len() as u16);
            try!(self.output(&buf));
        } else if value.len() <= MAX_BIN32 {
            let mut buf = [BIN32; U32_BYTES + 1];
            BigEndian::write_u32(&mut buf[1..], value.len() as u32);
            try!(self.output(&buf));
        } else {
            return Err(Error::simple(Reason::TooBig));
        }

        self.output(value)
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
        assert_eq!(::to_bytes(s).unwrap(), &[0xac, 0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x20,
                                             0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21]);
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
        assert_eq!(::to_bytes(v).unwrap(), &[0x94, 0x05, 0x08, 0x14, 0xcc, 0xe7]);
    }

    #[test]
    fn array16_test() {
        let v: Vec<isize> = vec![-5, 16, 101, -45, 184,
                                 89, 62, -233, -33, 304,
                                 76, 90, 23, 108, 45,
                                 -3, 2];
        assert_eq!(::to_bytes(v).unwrap(), &[0xdc,
                                             0x00, 0x11,
                                             0xfb,  0x10,  0x65,  0xd0, 0xd3,  0xcc, 0xb8,
                                             0x59,  0x3e,  0xd1, 0xff, 0x17,  0xd0, 0xdf,  0xd1, 0x01, 0x30,
                                             0x4c, 0x5a, 0x17, 0x6c, 0x2d,
                                             0xfd, 0x02]);
    }

    #[test]
    fn fixmap_test() {
        let mut map: BTreeMap<String, usize> = BTreeMap::new();
        map.insert("one".into(), 1);
        map.insert("two".into(), 2);
        map.insert("three".into(), 3);
        assert_eq!(::to_bytes(map).unwrap(), &[0x83,
                                               0xa3, 0x6f, 0x6e, 0x65,  0x01,
                                               0xa5, 0x74, 0x68, 0x72, 0x65, 0x65,  0x03,
                                               0xa3, 0x74, 0x77, 0x6f,  0x02]);
    }
}
