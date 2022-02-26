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
tl = "0.6.3"
# or, if you need SIMD
# (requires a nightly compiler!)
tl = { version = "0.6.3", features = ["simd"] }
```

The main function is `tl::parse()`. It accepts an HTML source code string and parses it. It is important to note that tl currently silently ignores tags that are invalid, sort of like browsers do. Sometimes, this means that large chunks of the HTML document do not appear in the resulting AST, although in the future this will likely be customizable, in case you need explicit error checking.

## Examples
Finding an element by its id attribute and printing the inner text:
```rust
let input = r#"<p id="text">Hello</p>"#;
let dom = tl::parse(input, tl::ParserOptions::default()).unwrap();
let parser = dom.parser();
let element = dom.get_element_by_id("text")
  .expect("Failed to find element")
  .get(parser)
  .unwrap();

assert_eq!(element.inner_text(parser), "Hello");
```

Finding a tag using the query selector API:
```rust
let input = r#"<div><img src="cool-image.png" /></div>"#;
let dom = tl::parse(input, tl::ParserOptions::default()).unwrap();
let img = dom.query_selector("img[src]").unwrap().next();
    
assert!(img.is_some());
```

Iterating over the subnodes of an HTML document:
```rust
let input = r#"<div><img src="cool-image.png" /></div>"#;
let dom = tl::parse(input, tl::ParserOptions::default()).unwrap();
let img = dom.nodes()
  .iter()
  .find(|node| {
    node.as_tag().map_or(false, |tag| tag.name() == "img")
  });
    
assert!(img.is_some());
```

Mutating the `href` attribute of an anchor tag:
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

## SIMD-accelerated parsing
This crate has optimized parsing functions which make use of SIMD. These are disabled by default and must be enabled explicitly by passing the `simd` feature flag due to the unstable feature `portable_simd`. This requires a **nightly** compiler!

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

## Design
Due to the nature of zero-copy parsers, the string must be kept alive for the entire lifetime of the parser/dom.
If this is not acceptable or simply not possible in your case, you can call `tl::parse_owned()`.
This goes through the same steps as `tl::parse()` but returns an `VDomGuard` instead of a `VDom`.
The difference is that `VDomGuard` carefully creates a self-referential struct in which it stores the input string, so you can keep the `VDomGuard` as long as you want and move it around as much as you want.
