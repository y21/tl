#[inline(never)]
pub fn is_ident(c: u8) -> bool {
    (b'0'..=b'9').contains(&c)
        || (b'A'..=b'Z').contains(&c)
        || (b'a'..=b'z').contains(&c)
        || c == b'-'
        || c == b'_'
}

#[inline(always)]
pub fn to_lower(byte: u8) -> u8 {
    let is_upper = (byte >= b'A' && byte <= b'Z') as u8;
    let lower = is_upper * 0x20;
    byte + lower
}
