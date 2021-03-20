pub fn is_ident(c: u8) -> bool {
    (c >= b'0' && c <= b'9')
        || (b'A'..=b'Z').contains(&c)
        || (b'a'..=b'z').contains(&c)
        || c == b'-'
        || c == b'_'
}
