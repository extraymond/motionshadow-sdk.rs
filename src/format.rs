use std::collections::HashMap;
use std::convert::TryInto;

const KEY_BYTE_LENGTH: usize = std::mem::size_of::<u32>();
const VALUE_REAL_BYTE_LENGTH: usize = std::mem::size_of::<f32>();
const VALUE_NONREAL_BYTE_LENGTH: usize = std::mem::size_of::<i16>();

pub struct Element<'a> {
    pub data: &'a [u8],
    pub real_value: bool,
}

#[derive(Debug, PartialEq)]
pub enum ValueArray {
    REAL(Vec<f32>),
    NONREAL(Vec<i16>),
}

impl<'a> Element<'a> {
    pub fn new(data: &'a [u8], length: i32, real_value: bool) -> Option<Self> {
        if (data.len() == length as usize) || (length == 0) {
            Some(Self { data, real_value })
        } else {
            log::debug!("invalid length to construct element");
            None
        }
    }
    pub fn get_data(&self, base: usize, length: usize, real_value: bool) -> Option<&[u8]> {
        let mut rv = None;
        if base + length <= self.data.len() {
            rv = Some(&self.data[base..(base + length)]);
        }
        rv
    }
}

pub fn read_config<'a>(
    data: &'a [u8],
    length: Option<usize>,
    real_value: bool,
    num_nodes: Option<usize>,
) -> HashMap<i32, ValueArray> {
    let mut rv: HashMap<i32, ValueArray> = HashMap::new();
    let idx_length = std::mem::size_of::<i32>();
    if let Some(len) = length {
        rv = data
            .chunks(idx_length + len * KEY_BYTE_LENGTH)
            .map(|chunk: &[u8]| {
                let key_slice: &[u8; KEY_BYTE_LENGTH] = chunk[0..idx_length]
                    .try_into()
                    .expect("to extract key slice");
                let key = i32::from_le_bytes(*key_slice);
                let value_slice: &[u8] = &chunk[idx_length..];
                let values = if real_value {
                    let values = value_slice
                        .chunks(VALUE_REAL_BYTE_LENGTH)
                        .map(|bytes: &[u8]| {
                            let size_bytes: &[u8; VALUE_REAL_BYTE_LENGTH] =
                                bytes.try_into().unwrap();
                            f32::from_le_bytes(*size_bytes)
                        })
                        .collect::<Vec<f32>>();
                    ValueArray::REAL(values)
                } else {
                    let values = value_slice
                        .chunks(VALUE_REAL_BYTE_LENGTH)
                        .map(|bytes: &[u8]| {
                            let size_bytes: &[u8; VALUE_NONREAL_BYTE_LENGTH] =
                                bytes.try_into().unwrap();
                            i16::from_le_bytes(*size_bytes)
                        })
                        .collect::<Vec<i16>>();
                    ValueArray::NONREAL(values)
                };
                (key, values)
            })
            .collect::<HashMap<i32, ValueArray>>();
    } else {
        let total_bytes = data.len();
        let chunk_size = total_bytes / num_nodes.unwrap();
        let value_chunk_size = chunk_size - 2 * KEY_BYTE_LENGTH;

        rv = data
            .chunks(chunk_size)
            .map(|chunk: &[u8]| {
                let key_slice: &[u8; KEY_BYTE_LENGTH] = chunk[0..idx_length].try_into().unwrap();
                let key = i32::from_le_bytes(*key_slice);

                let len_slice: &[u8; KEY_BYTE_LENGTH] =
                    chunk[idx_length..2 * idx_length].try_into().unwrap();
                let len = i32::from_le_bytes(*len_slice);

                let value_slice: &[u8] = &chunk[idx_length * 2..];
                let values = if real_value {
                    let values = value_slice
                        .chunks(VALUE_REAL_BYTE_LENGTH)
                        .map(|bytes: &[u8]| {
                            let size_bytes: &[u8; VALUE_REAL_BYTE_LENGTH] =
                                bytes.try_into().unwrap();
                            f32::from_le_bytes(*size_bytes)
                        })
                        .collect::<Vec<f32>>();
                    if value_chunk_size / VALUE_REAL_BYTE_LENGTH != values.len() {
                        log::warn!("invalid length");
                    }
                    ValueArray::REAL(values)
                } else {
                    let values = value_slice
                        .chunks(VALUE_REAL_BYTE_LENGTH)
                        .map(|bytes: &[u8]| {
                            let size_bytes: &[u8; VALUE_NONREAL_BYTE_LENGTH] =
                                bytes.try_into().unwrap();
                            i16::from_le_bytes(*size_bytes)
                        })
                        .collect::<Vec<i16>>();
                    if value_chunk_size / VALUE_NONREAL_BYTE_LENGTH != values.len() {
                        log::warn!("invalid length");
                    }

                    ValueArray::NONREAL(values)
                };
                (key, values)
            })
            .collect::<HashMap<i32, ValueArray>>();
    }
    rv
}
