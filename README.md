# tl
tl is a fast HTML parser written in pure Rust. <br />

- [Usage](#usage)
- [Examples](#examples)
- [SIMD-accelerated parsing](#simd-accelerated-parsing)
- [Benchmarks](#benchmarks)
- [Design](#design)

## Usage
Add `tl` to your dependencies.
```toml
[dependencies]
tl = "0.7.4"
# or, with explicit SIMD support
# (requires a nightly compiler!)
tl = { version = "0.7.4", features = ["simd"] }
```

The main function is `tl::parse()`. It accepts an HTML source code string and parses it. It is important to note that tl currently silently ignores tags that are invalid, sort of like browsers do. Sometimes, this means that large chunks of the HTML document do not appear in the resulting AST, although in the future this will likely be customizable, in case you need explicit error checking.

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

## Benchmarks
Results for parsing a ~320KB [HTML document](https://github.com/y21/rust-html-parser-benchmark/blob/80d24a260ab9377bc704aa0b12657539aeaa4777/data/wikipedia.html). Benchmarked using criterion on codespaces hardware.
```notrust
              time            thrpt
tl + simd     628.23 us       497.87 MiB/s
htmlstream    2.2786 ms       137.48 MiB/s
rusthtml      3.3881 ms       92.317 MiB/s
html5ever     5.7900 ms       54.021 MiB/s
rphtml        6.0154 ms       51.997 MiB/s
htmlparser    17.764 ms       17.608 MiB/s
```

[Source](https://github.com/y21/rust-html-parser-benchmark/tree/53238f68bbb57adc8dffdd245693ca1caa89cf4f)
