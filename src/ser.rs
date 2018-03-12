#[cfg(test)]
use alloc::btree_map::BTreeMap;
use alloc::string::String;
use constants::*;
use bytes::bytes;
use alloc::vec::Vec;
use value::{Value};
use io::{Writer, VecWriter};

pub trait Serializer {
    fn serialize_unsigned(&mut self, n: usize, major_type: u8);
    fn serialize_bytes(&mut self, bytes: Vec<u8>);
    fn serialize_seq(&mut self, len: usize);
    fn serialize_map(&mut self, len: usize);
    fn serialize_simple(&mut self, value: usize);
    fn serialize_string(&mut self, string: &String);
}

pub trait Serialize {
    fn serialize<S>(&mut self, serializer: &mut S) where S: Serializer;
}

pub struct VecSerializer<'a>  {
    pub writer: VecWriter<'a>,
}

pub fn to_bytes(mut value: Value) -> Vec<u8>
{
    let mut output = Vec::with_capacity(128);
    value.serialize(
        &mut VecSerializer::from_vec_writer(
            VecWriter{
                output: &mut output,
            }
        )
    );
    output
}

impl<'a> VecSerializer<'a> {
    fn from_vec_writer(vec_writer: VecWriter) -> VecSerializer {
        VecSerializer{writer: vec_writer}
    }

    fn encode_unsigned(&mut self, n: usize, major_type: u8) -> Vec<u8> {
        match n {
            n @ 0 ... 23 => vec![bytes::concat(major_type, n as u8)],
            n @ 24 ... MAX_U8 => vec![bytes::concat(major_type, 24), n as u8],
            _ => unreachable!()
        }
    }

}

impl<'a> Serializer for VecSerializer<'a> {
    fn serialize_unsigned(&mut self, n: usize, major_type: u8) {
        let bytes = self.encode_unsigned(n, major_type).to_vec();
        self.writer.write_bytes(bytes);
    }

    fn serialize_bytes(&mut self, bytes: Vec<u8>) {
        self.writer.write_bytes(bytes);
    }

    fn serialize_string(&mut self, string: &String) {
        self.serialize_unsigned(string.len(), 0b011);
        self.writer.write_bytes(string.as_bytes().to_vec());
    }

    fn serialize_seq(&mut self, len: usize) {
        self.serialize_unsigned(len, 0b100);
    }

    fn serialize_map(&mut self, len: usize) {
        self.serialize_unsigned(len, 0b101);
    }

    fn serialize_simple(&mut self, value: usize) {
        self.serialize_unsigned(value, 0b111);
    }
}

#[test]
fn serialize_map() {
    let mut test_map = BTreeMap::new();
    test_map.insert("key1".into(), Value::String("value1".into()));
    test_map.insert("key2".into(), Value::String("value2".into()));
    let value: Value = Value::Map(test_map);
    assert_eq!(vec![0xa2, 0x64, 0x6b, 0x65, 0x79, 0x31, 0x66, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x31, 0x64, 0x6b, 0x65, 0x79, 0x32, 0x66, 0x76, 0x61, 0x6c, 0x75, 0x65, 0x32], to_bytes(value));
}

#[test]
fn serialize_array() {
    let value = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert_eq!(vec![131, 1, 2, 3], to_bytes(value));
}

#[test]
fn serialize_string() {
    let value = Value::String("test".into());
    assert_eq!(vec![0x64, 0x74, 0x65, 0x73, 0x74], to_bytes(value));
}

#[test]
fn serialize_u8() {
    let value = Value::Int(2);
    assert_eq!(vec![2], to_bytes(value));
}

#[test]
fn serialize_u8_2() {
    let value = Value::Int(42);
    assert_eq!(vec![24, 42], to_bytes(value));
}

#[test]
fn serialize_null() {
    let value = Value::Null;
    assert_eq!(vec![246], to_bytes(value));
}
