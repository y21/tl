# tl
tl is an efficient and easy to use HTML parser written in Rust.<br />
It does minimal to no copying during parsing by borrowing parts of the input string.
Additionally, it keeps track of parsed elements and inserts elements with an `id` attribute
into an internal HashMap, which makes `get_element_by_id` as well as `get_elements_by_class_name` very fast.

## Examples
Finding an element by its `id` attribute and printing the inner text:
```rust
fn main() {
    let input = r#"<p id="text">Hello</p>"#;
    let dom = tl::parse(input);

    let element = dom.get_element_by_id("text").expect("Failed to find element");

    println!("Inner text: {}", element.inner_text());
}
```

-----
```toml
[dependencies]
tl = "0.1.0"
```

