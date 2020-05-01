use num::traits::{FromPrimitive, PrimInt};
use std::mem::size_of;

/// Convert a [`Vec`](std::vec::Vec)`<u8>` to a primitive int, in big-endian
/// format.
///
/// Unlike its friend, [`bytes_index`](crate::int_tools::bytes_index), this
/// will check the length of the Vec to ensure it is the correct length for the
/// int. It also will use the whole Vec instead of starting at an index.
///
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

/// Convert a `Vec<u8>` to a primitive int, beginning at the index specified.
///
/// Unlike [`bytes_to`], this will not check the length and will not attempt to
/// parse the entire Vec.
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

/// Return the next power of 2 greater than the number given.
/// If the number is 0, 0 will be returned.
pub fn next_power_2(len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let leading0s: u32 = len.leading_zeros();
    const TOTAL_ZEROS: u32 = usize::leading_zeros(0);
    (1 as usize) << (TOTAL_ZEROS - leading0s) as usize
}
