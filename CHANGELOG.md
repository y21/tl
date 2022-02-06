Changes annotated with `⚠` are breaking.

# 0.6.1
- Fixed an off-by-one error in the `QueryIterable` trait implementation for `HTMLTag` that caused query selectors on HTML tags to return one node less than they should.

# 0.6.0
> **Warning: This release contains breaking changes**
- ⚠ Removed deprecated method `VDom::find_node`
    - Alternative: use `VDom::nodes().iter().find(...)` instead
- ⚠ `Attributes::get()` now returns a reference to `Bytes` instead of cloning.
    - Prior to this version, it wasn't necessary to return a reference as the
      `Bytes` type was just an immutable `&[u8]`. Now it can hold owned data.
- ⚠ `HTMLTag::children()` no longer returns an iterator, and instead returns a wrapper struct around the children of the HTML tag.
    This wrapper struct makes it easy to obtain direct children of the tag (`Children::top()`),
    or all children (including their children, etc...) (`Children::all()`).
- ⚠ `Node::children()` no longer returns an iterator (see above).
- ⚠ `HTMLTag::name()` now returns a reference to `Bytes` instead of cloning (see above).
- Ability to create/parse query selectors independent of any parser (`tl::parse_query_selector`)
- Ability to reuse query selectors
- Ability to apply query selectors on `HTMLTag`s (see [#18](https://github.com/y21/tl/issues/18))
- `queryselector` module is now public
- `InnerNodeHandle` is now u32
- Remove unused `max_depth` parser option
- Add convenience `PartialEq<str>` and `PartialEq<[u8]>` impls for Bytes


# 0.5.0
> **Warning: This release contains breaking changes**
- Allow `Bytes` to store owned data through `Bytes::set()`
    - ⚠ The maximum length for `Bytes` is `u32::MAX`
- ⚠ `tl::parse()` now returns `Result<VDom<'a>, ParseError>`
- ⚠ `Attributes` fields are no longer public, instead use one of the provided methods
- ⚠ `HTMLTag::inner_html()` now takes a `&Parser` and no longer directly returns the substring
    - Node mutations to the tag or any of its subnodes means `inner_html` needs to be recomputed
    - Consider using `HTMLTag::raw()` if you never mutate any nodes

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
