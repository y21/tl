# tl
tl is an efficient and easy to use HTML parser written in Rust.<br />
It does minimal to no copying during parsing by borrowing parts of the input string.
Additionally, it keeps track of parsed elements and stores elements with an `id` attribute in an internal HashMap, which makes element lookups by ID/class name very fast.

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

## Usage
Add `tl` to your dependencies.
```toml
[dependencies]
tl = "0.1.0"
```

