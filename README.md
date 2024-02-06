# tl
tl is a fast HTML parser written in pure Rust. <br />

- [Usage](#usage)
- [Examples](#examples)
- [SIMD-accelerated parsing](#simd-accelerated-parsing)

This crate (currently) does *not* strictly follow the full specification of the HTML standard, however this usually is not a problem for most use cases. This crate generally attempts to support most "sane" HTML. Not being limited by a specification allows for more optimization opportunities.
If you need a parser that can (very quickly) parse the typical HTML document and you need a simple API to work with the DOM, give this crate a try.

If you need a parser that closely follows the standard, consider using [html5ever](https://github.com/servo/html5ever), [lol-html](https://github.com/cloudflare/lol-html), or [html5gum](https://github.com/untitaker/html5gum).

## Usage
Add `tl` to your dependencies.
```toml
[dependencies]
tl = "0.7.8"
# or, with explicit SIMD support
# (requires a nightly compiler!)
tl = { version = "0.7.8", features = ["simd"] }
```

The main function is `tl::parse()`. It accepts an HTML source code string and parses it. It is important to note that tl currently silently ignores tags that are invalid, sort of like browsers do. Sometimes, this means that large chunks of the HTML document do not appear in the resulting tree.

```rust
let dom = tl::parse(r#"<p id="text">Hello</p>"#, tl::ParserOptions::default()).unwrap();
let parser = dom.parser();
let element = dom.get_element_by_id("text")
  .expect("Failed to find element")
  .get(parser)
  .unwrap();

assert_eq!(element.inner_text(parser), "Hello");
```

## Examples
<details>
  <summary>Finding a tag using the query selector API</summary>

```rust
let dom = tl::parse(r#"<div><img src="cool-image.png" /></div>"#, tl::ParserOptions::default()).unwrap();
let img = dom.query_selector("img[src]").unwrap().next();
    
assert!(img.is_some());
```
</details>

<details>
  <summary>Iterating over the subnodes of an HTML document</summary>

```rust
let dom = tl::parse(r#"<div><img src="cool-image.png" /></div>"#, tl::ParserOptions::default()).unwrap();
let img = dom.nodes()
  .iter()
  .find(|node| {
    node.as_tag().map_or(false, |tag| tag.name() == "img")
  });
    
assert!(img.is_some());
```
</details>

<details>
  <summary>Mutating the `href` attribute of an anchor tag:</summary>

> In a real world scenario, you would want to handle errors properly instead of unwrapping.
```rust
let input = r#"<div><a href="/about">About</a></div>"#;
let mut dom = tl::parse(input, tl::ParserOptions::default())
  .expect("HTML string too long");
  
let anchor = dom.query_selector("a[href]")
  .expect("Failed to parse query selector")
  .next()
  .expect("Failed to find anchor tag");

let parser_mut = dom.parser_mut();

let anchor = anchor.get_mut(parser_mut)
  .expect("Failed to resolve node")
  .as_tag_mut()
  .expect("Failed to cast Node to HTMLTag");

let attributes = anchor.attributes_mut();

attributes.get_mut("href")
  .flatten()
  .expect("Attribute not found or malformed")
  .set("http://localhost/about");

assert_eq!(attributes.get("href").flatten(), Some(&"http://localhost/about".into()));
```
</details>


## SIMD-accelerated parsing
This crate has utility functions used by the parser which make use of SIMD (e.g. finding a specific byte by looking at the next 16 bytes at once, instead of going through the string one by one). These are disabled by default and must be enabled explicitly by passing the `simd` feature flag due to the unstable feature `portable_simd`. This requires a **nightly** compiler!

If the `simd` feature is not enabled, it will fall back to stable alternatives that don't explicitly use SIMD intrinsics, but are still decently well optimized, using techniques such as manual loop unrolling to remove boundary checks and other branches by a factor of 16, which also helps LLVM further optimize the code and potentially generate SIMD instructions by itself.
