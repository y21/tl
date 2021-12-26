# 0.4.4
- Parse unquoted attribute values properly (`<a href=foo>`) [#12]
- Parse valueless attributes properly (`<iframe allowfullscreen>`) [#11]

# 0.4.3
- Add optional `simd` feature flag for SIMD-accelerated parsing

# 0.4.2
- Keep track of recursion depth when parsing to avoid overflowing the stack

# 0.4.1
- Fixed an infinite loop bug in `parse_single` and a panic that occurs when trying to parse incomplete markup tags (`<!`)

# 0.4.0
- Remove `AsBytes` trait in favor of `From<T> for Bytes`
- Add `VDom::query_selector()`
- Introduce `InnerHandleNode` to hide implementation details of `NodeHandle`
- Make `Attributes::is_class_member()` generic over `T: Into<Bytes<'a>>`
- Add `Attributes::get_attribute()`
- Make `HTMLTag::name()` and `HTMLTag::inner_html()` return owned `Bytes`
- 

# 0.3.0
- Deprecate `VDom::find_node` in favor of `VDom::nodes().iter().find()`
