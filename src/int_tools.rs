#[inline]
pub fn bytes_to_u16(bytes: &Vec<u8>) -> u16 {
    assert_eq!(bytes.len(), 2);
    (bytes[1] as u16) << 8 + (bytes[0] as u16)
}

#[inline]
pub fn bytes_to_u32(bytes: &Vec<u8>) -> u32 {
    assert_eq!(bytes.len(), 4);
    (bytes[0] as u32)
        + (bytes[1] as u32) << 8
        + (bytes[2] as u32) << 16
        + (bytes[3] as u32) << 24
}
