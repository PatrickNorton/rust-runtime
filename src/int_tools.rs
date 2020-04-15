use num_traits::{FromPrimitive, PrimInt};
use std::mem::size_of;

#[inline]
pub fn bytes_to<T>(bytes: &Vec<u8>) -> T
where
    T: PrimInt + FromPrimitive,
{
    let byte_size = size_of::<T>();
    assert_eq!(bytes.len(), byte_size);
    let mut result: T = T::zero();
    for i in 0..byte_size {
        let bytes_i: T = FromPrimitive::from_u8(bytes[i]).unwrap();
        result = result | bytes_i << (byte_size - 1 - i) * 8
    }
    result
}

pub fn bytes_index<T>(bytes: &Vec<u8>, index: &mut usize) -> T
where
    T: PrimInt + FromPrimitive,
{
    let byte_size = size_of::<T>();
    let mut result: T = T::zero();
    for i in 0..byte_size {
        let bytes_i: T = FromPrimitive::from_u8(bytes[i + *index]).unwrap();
        result = result | bytes_i << (byte_size - 1 - i) * 8
    }
    *index += byte_size;
    result
}
