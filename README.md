# tl
tl is a very fast, zero-copy HTML parser written in pure Rust. <br />

## Design
Due to the zero-copy nature of parsers, the string must be kept alive for the entire lifetime of the parser/dom.
If this is not acceptable or simply not possible in your case, you can call `tl::parse_owned()`.
This goes through the same steps as `tl::parse()` but returns an `OwnedVDom` instead of a `VDom`.
The difference is that `OwnedVDom` carefully creates a self-referential struct in which it stores the input string, so you can keep the `OwnedVDom` as long as you want and move it around as much as you want.

### `InlineVec` and `InlineHashMap`
`InlineVec` is a wrapper around an array/vec capable of storing a small number of elements on the stack, and if it becomes too large it will move all elements to the heap. 
This library makes use of this concept in multiple places. "Children" of nodes (subnodes) are stored in an `InlineVec`, because it turns out that very often HTML tags don't actually have that many direct subnodes. This has the advantage that most of the time, HTML tags are mostly entirely allocated on the stack.
`InlineHashMap` is very similar to `InlineVec`, except that for a small collection, it actually stores the `(Key, Value)` pair in a regular stack-allocated array, and traverses this array when doing a key lookup without doing any sort of hashing. Turns out that hashing keys only pays off for larger collections. If the length is small, hashing may actually be slower than just going through a few elements.

### `Bytes`
Some functions return a `Bytes` struct, which is an internal struct that is used to borrow a part of the input string.
This is mainly used over a raw `&[u8]` for its `Debug` implementation.

## Usage
The main function is `tl::parse()`. It accepts an HTML source code string and parses it. It is important to note that tl currently silently ignores tags that are invalid, sort of like browsers do. Sometimes, this means that large chunks of the HTML document do not appear in the resulting AST, although in the future this will likely be customizable, in case you need explicit error checking.

Finding an element by its id attribute and printing the inner text:
```rust
fn main() {
    let input = r#"<p id="text">Hello</p>"#;
    let dom = tl::parse(input);
    let parser = dom.parser();
    let element = dom.get_element_by_id("text")
        .expect("Failed to find element")
        .get(parser)
        .unwrap();

    println!("Inner text: {}", element.inner_text(parser));
}
```

Iterating over the subnodes of an HTML document:
```rust
fn main() {
    let input = r#"<div><img src="cool-image.png" /></div>"#;
    let dom = tl::parse(input);
    let img = dom.nodes()
        .iter()
        .find(|node| {
            node.as_tag().map_or(false, |tag| tag.name() == &"img".into())
        });
    
    println!("{:?}", img);
}
```


## Usage
Add `tl` to your dependencies.
```toml
[dependencies]
tl = "0.3.0"
```