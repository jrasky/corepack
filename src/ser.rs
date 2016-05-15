use std::mem;
use std::result;

use byteorder::{ByteOrder, BigEndian};

use serde;

use defs::*;
use error::*;

pub type Result = result::Result<(), Error>;

pub struct Serializer<F: FnMut(&[u8]) -> Result> {
    output: F
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

    fn serialize_bool(&mut self, v: bool) -> Result {
        if v {
            self.output(&[TRUE])
        } else {
            self.output(&[FALSE])
        }
    }

    fn serialize_i64(&mut self, value: i64) -> Result {
        if value >= FIXINT_MIN as i64 && value < 0 {
            self.output(&[unsafe {mem::transmute(value as i8)}])
        } else if value >= i8::min_value() as i64 && value <= i8::max_value() as i64 {
            self.output(&[INT8, unsafe {mem::transmute(value as i8)}])
        } else if value >= i16::min_value() as i64 && value <= i16::max_value() as i64 {
            let mut buf = [INT16; U16_BYTES + 1];
            BigEndian::write_i16(&mut buf[1..], value as i16);
            self.output(&buf)
        } else if value >= i32::min_value() as i64 && value <= i32::max_value() as i64 {
            let mut buf = [INT32; U32_BYTES + 1];
            BigEndian::write_i32(&mut buf[1..], value as i32);
            self.output(&buf)
        } else {
            let mut buf = [INT64; U64_BYTES + 1];
            BigEndian::write_i64(&mut buf[1..], value);
            self.output(&buf)
        }
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
            try!(self.output(&[value.len() as u8 & FIXSTR_MASK]));
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

    fn serialize_unit(&mut self) -> Result {
        self.output(&[NIL])
    }

    fn serialize_none(&mut self) -> Result {
        self.serialize_unit()
    }

    fn serialize_some<V>(&mut self, value: V) -> Result
        where V: serde::Serialize {
        value.serialize(self)
    }

    fn serialize_seq<V>(&mut self, mut visitor: V) -> Result
        where V: serde::ser::SeqVisitor {

        if let Some(size) = visitor.len() {
            if size <= MAX_FIXARRAY {
                try!(self.output(&[size as u8 & FIXARRAY_MASK]));
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
        } else {
            return Err(Error::simple(Reason::Unsized));
        }

        loop {
            match visitor.visit(self) {
                Ok(Some(())) => {},
                Ok(None) => {
                    break;
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    fn serialize_seq_elt<T>(&mut self, value: T) -> Result
        where T: serde::Serialize {
        value.serialize(self)
    }

    fn serialize_map<V>(&mut self, mut visitor: V) -> Result
        where V: serde::ser::MapVisitor {
        if let Some(size) = visitor.len() {
            if size <= MAX_FIXMAP {
                try!(self.output(&[size as u8 & FIXMAP_MASK]));
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
        } else {
            return Err(Error::simple(Reason::Unsized));
        }

        loop {
            match visitor.visit(self) {
                Ok(Some(())) => {},
                Ok(None) => {
                    break;
                },
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result 
        where K: serde::Serialize, V: serde::Serialize {
        try!(key.serialize(self));
        value.serialize(self)
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
