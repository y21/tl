pub fn is_ident(c: u8) -> bool {
    (c >= b'0' && c <= b'9') || (c >= b'A' && c <= b'Z') || (c >= b'a' && c <= b'z') || c == b'-' || c == b'_'
}