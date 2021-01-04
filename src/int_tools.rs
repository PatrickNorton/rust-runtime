use std::char;
use std::convert::TryInto;
use std::mem::size_of;

pub trait FromBytes {
    fn from_be(bytes: &[u8]) -> Self;
    fn from_le(bytes: &[u8]) -> Self;
}

macro_rules! impl_from_bytes {
    ($type: ty) => {
        impl FromBytes for $type {
            #[inline]
            fn from_be(bytes: &[u8]) -> Self {
                <$type>::from_be_bytes(bytes.try_into().unwrap())
            }

            #[inline]
            fn from_le(bytes: &[u8]) -> Self {
                <$type>::from_le_bytes(bytes.try_into().unwrap())
            }
        }
    };
}

impl_from_bytes!(u8);
impl_from_bytes!(i8);
impl_from_bytes!(u16);
impl_from_bytes!(i16);
impl_from_bytes!(u32);
impl_from_bytes!(i32);
impl_from_bytes!(u64);
impl_from_bytes!(i64);
impl_from_bytes!(u128);
impl_from_bytes!(i128);
impl_from_bytes!(usize);
impl_from_bytes!(isize);

impl FromBytes for char {
    fn from_be(bytes: &[u8]) -> Self {
        let val = u32::from_be_bytes(bytes.try_into().unwrap());
        char::from_u32(val).expect(&*format!("Invalid char value {}", val))
    }

    fn from_le(bytes: &[u8]) -> Self {
        let val = u32::from_le_bytes(bytes.try_into().unwrap());
        char::from_u32(val).expect(&*format!("Invalid char value {}", val))
    }
}

/// Convert a `Vec<u8>` to a primitive int, beginning at the index specified.
///
/// Unlike [`bytes_to`], this will not check the length and will not attempt to
/// parse the entire Vec.
pub fn bytes_index<T>(bytes: &[u8], index: &mut usize) -> T
where
    T: FromBytes,
{
    let byte_size = size_of::<T>();
    let result = T::from_be(&bytes[*index..*index + byte_size]);
    *index += byte_size;
    result
}

/// Convert a `Vec<u8>` to a primitive int, beginning at the index specified.
///
/// Unlike [`bytes_to`], this will not check the length and will not attempt to
/// parse the entire Vec.
pub fn bytes_index_le<T>(bytes: &[u8], index: &mut usize) -> T
where
    T: FromBytes,
{
    let byte_size = size_of::<T>();
    let result = T::from_le(&bytes[*index..*index + byte_size]);
    *index += byte_size;
    result
}

/// Return the next power of 2 greater than the number given.
/// If the number is 0, 0 will be returned.
pub fn next_power_2(len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let leading0s = len.leading_zeros();
    const TOTAL_ZEROS: u32 = usize::leading_zeros(0);
    1_usize << (TOTAL_ZEROS - leading0s)
}
