use std::ops::{Deref, DerefMut};
use std::iter::Iterator;

use alloc::boxed::Box;

use collections::{String, Vec};

use serde::{Serialize, Deserialize, Serializer, Deserializer, Error};
use serde::ser::impls::{SeqIteratorVisitor, MapIteratorVisitor};
use serde::de::value::ValueDeserializer;

use serde::{ser, de, bytes};

use error;

#[derive(Debug, Clone)]
pub enum Generic {
    Nil,
    False,
    True,
    Int(i64),
    UInt(u64),
    F32(f32),
    F64(f64),
    Bin(Box<[u8]>),
    Str(Box<str>),
    Array(Box<[Generic]>),
    Map(Box<[(Generic, Generic)]>),
    Ext(i8, Box<[u8]>)
}

struct SeqVisitor<I: Iterator<Item=Generic>> {
    iter: I
}

struct MapVisitor<I: Iterator<Item=(Generic, Generic)>> {
    iter: I,
    value: Option<Generic>
}

struct ExtVisitor<'a> {
    ty: i8,
    state: u8,
    data: &'a [u8]
}

struct MapGeneric(Vec<(Generic, Generic)>);
struct VecGeneric(Vec<Generic>);

pub struct GenericVisitor;

impl<I: Iterator<Item=Generic>> de::SeqVisitor for SeqVisitor<I> {
    type Error = error::Error;

    fn visit<T>(&mut self) -> Result<Option<T>, error::Error> where T: Deserialize {
        if let Some(mut item) = self.iter.next() {
            Ok(Some(try!(T::deserialize(&mut item))))
        } else {
            Ok(None)
        }
    }

    fn end(&mut self) -> Result<(), error::Error> {
        if self.iter.next().is_none() {
            Ok(())
        } else {
            Err(de::Error::invalid_length(self.size_hint().0))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I: Iterator<Item=(Generic, Generic)>> de::MapVisitor for MapVisitor<I> {
    type Error = error::Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>, error::Error> where K: Deserialize {
        let item;

        if let Some(next) = self.iter.next() {
            item = next;
        } else {
            return Ok(None);
        }

        let (mut key, value) = item;

        self.value = Some(value);
        Ok(Some(try!(K::deserialize(&mut key))))
    }

    fn visit_value<V>(&mut self) -> Result<V, error::Error> where V: Deserialize {
        if let Some(mut value) = self.value.take() {
            Ok(try!(V::deserialize(&mut value)))
        } else {
            Err(de::Error::end_of_stream())
        }
    }

    fn visit<K, V>(&mut self) -> Result<Option<(K, V)>, error::Error> where K: Deserialize, V: Deserialize {
        if let Some((mut key, mut value)) = self.iter.next() {
            Ok(Some((try!(K::deserialize(&mut key)), try!(V::deserialize(&mut value)))))
        } else {
            Ok(None)
        }
    }

    fn end(&mut self) -> Result<(), error::Error> {
        if self.iter.next().is_none() {
            Ok(())
        } else {
            Err(de::Error::invalid_length(self.size_hint().0))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a> ser::MapVisitor for ExtVisitor<'a> {
    fn visit<S>(&mut self, s: &mut S) -> Result<Option<()>, S::Error> where S: Serializer {
        if self.state == 0 {
            self.state += 1;
            s.serialize_struct_elt("type", self.ty).map(|ok| Some(ok))
        } else if self.state == 1 {
            self.state += 1;
            s.serialize_struct_elt("data", self.data).map(|ok| Some(ok))
        } else {
            Ok(None)
        }
    }

    fn len(&self) -> Option<usize> {
        Some(2)
    }
}

impl<'a> de::MapVisitor for ExtVisitor<'a> {
    type Error = error::Error;

    fn visit_key<T>(&mut self) -> Result<Option<T>, error::Error> where T: Deserialize {
        if self.state == 0 {
            let mut de = ValueDeserializer::<error::Error>::into_deserializer("type");
            Ok(Some(try!(T::deserialize(&mut de))))
        } else if self.state == 1 {
            let mut de = ValueDeserializer::<error::Error>::into_deserializer("data");
            Ok(Some(try!(T::deserialize(&mut de))))
        } else {
            Ok(None)
        }
    }

    fn visit_value<T>(&mut self) -> Result<T, error::Error> where T: Deserialize {
        if self.state == 0 {
            self.state += 1;
            let mut de = ValueDeserializer::<error::Error>::into_deserializer(self.ty);
            Ok(try!(T::deserialize(&mut de)))
        } else if self.state == 1 {
            self.state += 1;
            let mut de = ValueDeserializer::<error::Error>::into_deserializer(bytes::Bytes::from(self.data));
            Ok(try!(T::deserialize(&mut de)))
        } else {
            Err(de::Error::end_of_stream())
        }
    }

    fn end(&mut self) -> Result<(), error::Error> {
        if self.state > 1 {
            Ok(())
        } else {
            Err(de::Error::invalid_length(2 - self.state as usize))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (2 - self.state as usize, Some(2 - self.state as usize))
    }
}

impl Deref for MapGeneric {
    type Target = Vec<(Generic, Generic)>;

    fn deref(&self) -> &Vec<(Generic, Generic)> {
        &self.0
    }
}

impl DerefMut for MapGeneric {
    fn deref_mut(&mut self) -> &mut Vec<(Generic, Generic)> {
        &mut self.0
    }
}

impl Deref for VecGeneric {
    type Target = Vec<Generic>;

    fn deref(&self) -> &Vec<(Generic)> {
        &self.0
    }
}

impl DerefMut for VecGeneric {
    fn deref_mut(&mut self) -> &mut Vec<Generic> {
        &mut self.0
    }
}

impl GenericVisitor {
    pub fn visit_ext<E>(&mut self, ty: i8, data: Vec<u8>) -> Result<Generic, E> where E: Error {
        Ok(Generic::new_ext(ty, data.into_boxed_slice()))
    }
}

impl de::Visitor for GenericVisitor {
    type Value = Generic;

    fn visit_bool<E>(&mut self, v: bool) -> Result<Generic, E> where E: Error {
        if v {
            Ok(Generic::True)
        } else {
            Ok(Generic::False)
        }
    }

    fn visit_i64<E>(&mut self, v: i64) -> Result<Generic, E> where E: Error {
        Ok(Generic::Int(v))
    }

    fn visit_u64<E>(&mut self, v: u64) -> Result<Generic, E> where E: Error {
        Ok(Generic::UInt(v))
    }

    fn visit_f32<E>(&mut self, v: f32) -> Result<Generic, E> where E: Error {
        Ok(Generic::F32(v))
    }

    fn visit_f64<E>(&mut self, v: f64) -> Result<Generic, E> where E: Error {
        Ok(Generic::F64(v))
    }

    fn visit_str<E>(&mut self, v: &str) -> Result<Generic, E> where E: Error {
        Ok(Generic::Str(String::from(v).into_boxed_str()))
    }

    fn visit_string<E>(&mut self, v: String) -> Result<Generic, E> where E: Error {
        Ok(Generic::Str(v.into_boxed_str()))
    }

    fn visit_unit<E>(&mut self) -> Result<Generic, E> where E: Error {
        Ok(Generic::Nil)
    }

    fn visit_none<E>(&mut self) -> Result<Generic, E> where E: Error {
        self.visit_unit()
    }

    fn visit_some<D>(&mut self, d: &mut D) -> Result<Generic, D::Error> where D: Deserializer {
        d.deserialize(GenericVisitor)
    }

    fn visit_newtype_struct<D>(&mut self, d: &mut D) -> Result<Generic, D::Error> where D: Deserializer {
        d.deserialize(GenericVisitor)
    }

    fn visit_map<V>(&mut self, mut v: V) -> Result<Generic, V::Error> where V: de::MapVisitor {
        let mut buf = vec![];

        while let Some(pair) = try!(v.visit::<Generic, Generic>()) {
            buf.push(pair);
        }

        Ok(Generic::Map(buf.into_boxed_slice()))
    }

    fn visit_seq<V>(&mut self, mut v: V) -> Result<Generic, V::Error> where V: de::SeqVisitor {
        let mut buf = vec![];

        while let Some(item) = try!(v.visit::<Generic>()) {
            buf.push(item);
        }

        Ok(Generic::Array(buf.into_boxed_slice()))
    }

    fn visit_bytes<E>(&mut self, v: &[u8]) -> Result<Generic, E> where E: Error {
        Ok(Generic::Bin(Vec::from(v).into_boxed_slice()))
    }

    fn visit_byte_buf<E>(&mut self, v: Vec<u8>) -> Result<Generic, E> where E: Error {
        Ok(Generic::Bin(v.into_boxed_slice()))
    }
}

impl Serialize for Generic {
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error> where S: Serializer {
        use self::Generic::*;

        match self {
            &Nil => s.serialize_unit(),
            &False => s.serialize_bool(false),
            &True => s.serialize_bool(true),
            &Int(i) => s.serialize_i64(i),
            &UInt(i) => s.serialize_u64(i),
            &F32(f) => s.serialize_f32(f),
            &F64(f) => s.serialize_f64(f),
            &Bin(ref b) => s.serialize_bytes(b),
            &Str(ref st) => s.serialize_str(st),
            &Array(ref a) => s.serialize_seq(SeqIteratorVisitor::new(
                a.iter(), Some(a.len())
            )),
            &Map(ref m) => s.serialize_map(MapIteratorVisitor::new(
                m.iter().cloned(), Some(m.len())
            )),
            &Ext(ty, ref data) => s.serialize_struct("Ext", ExtVisitor {
                ty: ty,
                state: 0,
                data: data
            })
        }
    }
}

impl Deserialize for Generic {
    fn deserialize<D>(d: &mut D) -> Result<Generic, D::Error> where D: Deserializer {
        d.deserialize(GenericVisitor)
    }
}

impl de::Deserializer for Generic {
    type Error = error::Error;

    fn deserialize<V>(&mut self, mut v: V) -> Result<V::Value, error::Error> where V: de::Visitor {
        use self::Generic::*;

        match self {
            &mut Nil => v.visit_unit(),
            &mut False => v.visit_bool(false),
            &mut True => v.visit_bool(true),
            &mut Int(i) => v.visit_i64(i),
            &mut UInt(i) => v.visit_u64(i),
            &mut F32(f) => v.visit_f32(f),
            &mut F64(f) => v.visit_f64(f),
            &mut Bin(ref b) => v.visit_bytes(&b),
            &mut Str(ref s) => v.visit_str(&s),
            &mut Array(ref a) => v.visit_seq(SeqVisitor {
                iter: a.iter().cloned()
            }),
            &mut Map(ref m) => v.visit_map(MapVisitor {
                iter: m.iter().cloned(),
                value: None
            }),
            &mut Ext(ty, ref data) => v.visit_map(ExtVisitor {
                ty: ty,
                state: 0,
                data: data
            })
        }
    }
}

impl ser::Serializer for MapGeneric {
    type Error = error::Error;

    fn serialize_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), error::Error> where K: Serialize, V: Serialize {
        let mut buf = VecGeneric(vec![]);

        try!(value.serialize(&mut buf));
        try!(key.serialize(&mut buf));

        if buf.len() != 2 {
            return Err(ser::Error::invalid_value("Key and Value did not serialize to exactly one item each"));
        }

        let key = buf.pop().unwrap();
        let value = buf.pop().unwrap();

        self.push((key, value));

        Ok(())
    }

    fn serialize_bool(&mut self, _: bool) -> Result<(), error::Error> {
        Err(error::Error::simple(error::Reason::BadType))
    }

    fn serialize_i64(&mut self, _: i64) -> Result<(), error::Error> {
        Err(error::Error::simple(error::Reason::BadType))
    }

    fn serialize_u64(&mut self, _: u64) -> Result<(), error::Error> {
        Err(error::Error::simple(error::Reason::BadType))
    }

    fn serialize_f64(&mut self, _: f64) -> Result<(), error::Error> {
        Err(error::Error::simple(error::Reason::BadType))
    }

    fn serialize_str(&mut self, _: &str) -> Result<(), error::Error> {
        Err(error::Error::simple(error::Reason::BadType))
    }

    fn serialize_unit(&mut self) -> Result<(), error::Error> {
        Err(error::Error::simple(error::Reason::BadType))
    }

    fn serialize_none(&mut self) -> Result<(), error::Error> {
        Err(error::Error::simple(error::Reason::BadType))
    }

    fn serialize_some<V>(&mut self, _: V) -> Result<(), error::Error> where V: Serialize {
        Err(error::Error::simple(error::Reason::BadType))
    }

    fn serialize_seq<V>(&mut self, _: V) -> Result<(), error::Error> where V: ser::SeqVisitor {
        Err(error::Error::simple(error::Reason::BadType))
    }

    fn serialize_seq_elt<T>(&mut self, _: T) -> Result<(), error::Error> where T: Serialize {
        Err(error::Error::simple(error::Reason::BadType))
    }

    fn serialize_map<V>(&mut self, _: V) -> Result<(), error::Error> where V: ser::MapVisitor {
        Err(error::Error::simple(error::Reason::BadType))
    }
}

impl ser::Serializer for VecGeneric {
    type Error = error::Error;

    fn serialize_bool(&mut self, v: bool) -> Result<(), error::Error> {
        if v {
            self.push(Generic::True);
        } else {
            self.push(Generic::False);
        }

        Ok(())
    }

    fn serialize_i64(&mut self, v: i64) -> Result<(), error::Error> {
        self.push(Generic::Int(v));

        Ok(())
    }

    fn serialize_u64(&mut self, v: u64) -> Result<(), error::Error> {
        self.push(Generic::UInt(v));

        Ok(())
    }

    fn serialize_f32(&mut self, f: f32) -> Result<(), error::Error> {
        self.push(Generic::F32(f));

        Ok(())
    }

    fn serialize_f64(&mut self, f: f64) -> Result<(), error::Error> {
        self.push(Generic::F64(f));

        Ok(())
    }

    fn serialize_str(&mut self, value: &str) -> Result<(), error::Error> {
        self.push(Generic::Str(String::from(value).into_boxed_str()));

        Ok(())
    }

    fn serialize_unit(&mut self) -> Result<(), error::Error> {
        self.push(Generic::Nil);

        Ok(())
    }

    fn serialize_none(&mut self) -> Result<(), error::Error> {
        self.serialize_unit()
    }

    fn serialize_some<V>(&mut self, value: V) -> Result<(), error::Error> where V: Serialize {
        value.serialize(self)
    }

    fn serialize_seq<V>(&mut self, mut visitor: V) -> Result<(), error::Error> where V: ser::SeqVisitor {
        let mut buf = VecGeneric(vec![]);

        while try!(visitor.visit(&mut buf)).is_some() {}

        self.push(Generic::Array(buf.0.into_boxed_slice()));

        Ok(())
    }

    fn serialize_seq_elt<T>(&mut self, value: T) -> Result<(), error::Error> where T: Serialize {
        value.serialize(self)
    }

    fn serialize_map<V>(&mut self, mut visitor: V) -> Result<(), error::Error> where V: ser::MapVisitor {
        let mut buf = MapGeneric(vec![]);

        while try!(visitor.visit(&mut buf)).is_some() {}

        self.push(Generic::Map(buf.0.into_boxed_slice()));

        Ok(())
    }

    fn serialize_map_elt<K, V>(&mut self, _: K, _: V) -> Result<(), error::Error> where K: Serialize, V: Serialize {
        Err(error::Error::simple(error::Reason::BadType))
    }
}

impl Generic {
    pub fn from_value<V>(value: V) -> Result<Generic, error::Error> where V: Serialize {
        let mut buf = VecGeneric(vec![]);

        try!(value.serialize(&mut buf));

        if let Some(generic) = buf.pop() {
            if !buf.is_empty() {
                Err(error::Error::new(error::Reason::BadLength, "Value serialized into more than one item".into()))
            } else {
                Ok(generic)
            }
        } else {
            Err(error::Error::new(error::Reason::BadLength, "Value serialized into no items".into()))
        }
    }

    pub fn serialize_pack<F>(&self, s: &mut ::ser::Serializer<F>) -> Result<(), error::Error>
        where F: FnMut(&[u8]) -> Result<(), error::Error> {
        match self {
            &Generic::Ext(ty, ref data) => s.serialize_ext(ty, &data),
            value => Serialize::serialize(value, s)
        }
    }

    pub fn new_ext(ty: i8, data: Box<[u8]>) -> Generic {
        Generic::Ext(ty, data)
    }

    pub fn is_nil(&self) -> bool {
        if let &Generic::Nil = self {
            true
        } else {
            false
        }
    }

    pub fn is_false(&self) -> bool {
        if let &Generic::False = self {
            true
        } else {
            false
        }
    }

    pub fn is_true(&self) -> bool {
        if let &Generic::True = self {
            true
        } else {
            false
        }
    }

    pub fn is_int(&self) -> bool {
        if let &Generic::Int(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_uint(&self) -> bool {
        if let &Generic::UInt(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_f32(&self) -> bool {
        if let &Generic::F32(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_f64(&self) -> bool {
        if let &Generic::F64(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_bin(&self) -> bool {
        if let &Generic::Bin(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_str(&self) -> bool {
        if let &Generic::Str(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_array(&self) -> bool {
        if let &Generic::Array(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_map(&self) -> bool {
        if let &Generic::Map(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_ext(&self) -> bool {
        if let &Generic::Ext(_, _) = self {
            true
        } else {
            false
        }
    }

    pub fn ext_type(&self) -> Option<i8> {
        if let &Generic::Ext(ty, _) = self {
            Some(ty)
        } else {
            None
        }
    }

    pub fn ext_data(&self) -> Option<&[u8]> {
        if let &Generic::Ext(_, ref data) = self {
            Some(&data)
        } else {
            None
        }
    }
}
