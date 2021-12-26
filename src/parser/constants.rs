pub const OPENING_TAG: u8 = b'<';
pub const END_OF_TAG: [u8; 2] = [b'<', b'/'];
pub const SELF_CLOSING: [u8; 2] = [b'/', b'>'];
pub const COMMENT: &[u8; 2] = b"--";
pub const ID_ATTR: &[u8; 2] = b"id";
pub const CLASS_ATTR: &[u8; 5] = b"class";
pub const VOID_TAGS: &[&[u8]; 15] = &[
    b"area", b"base", b"br", b"col", b"embed", b"hr", b"img", b"input", b"keygen", b"link",
    b"meta", b"param", b"source", b"track", b"wbr",
];
