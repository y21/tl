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
Using `VDom::find_node()` to dynamically find a subnode.
> Note: If the HTML tag has an `id` attribute that you can use,
> you probably want to use `get_element_by_id` instead, as it does not iterate over the tree to find the element.
```rs
fn main() {
    let input = r#"<div><img src="cool-image.png" /></div>"#;
    let dom = tl::parse(input);
    let element = dom.find_node(|node| {
        node.as_tag()
            .unwrap()
            .attributes()
            .raw
            .contains_key(&"src".into())
    });
    println!("{:?}", element);
}
```


## Usage
Add `tl` to your dependencies.
```toml
[dependencies]
tl = "0.2.1"
```

