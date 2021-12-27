# tl
tl is a fast HTML parser written in pure Rust. <br />

- [Usage](#usage)
- [SIMD-accelerated parsing](#simd-accelerated-parsing)
- [Benchmarks](#benchmarks)
- [Design](#design)

## Usage
Add `tl` to your dependencies.
```toml
[dependencies]
tl = "0.4.4"
# or, if you need SIMD
tl = { version = "0.4.4", features = ["simd"] }
```

The main function is `tl::parse()`. It accepts an HTML source code string and parses it. It is important to note that tl currently silently ignores tags that are invalid, sort of like browsers do. Sometimes, this means that large chunks of the HTML document do not appear in the resulting AST, although in the future this will likely be customizable, in case you need explicit error checking.

Finding an element by its id attribute and printing the inner text:
```rust
fn main() {
    let input = r#"<p id="text">Hello</p>"#;
    let dom = tl::parse(input, tl::ParserOptions::default());
    let parser = dom.parser();
    let element = dom.get_element_by_id("text")
        .expect("Failed to find element")
        .get(parser)
        .unwrap();

    println!("Inner text: {}", element.inner_text(parser));
}
```

Finding a tag using the query selector API:
```rust
fn main() {
    let input = r#"<div><img src="cool-image.png" /></div>"#;
    let dom = tl::parse(input, tl::ParserOptions::default());
    let img = dom.query_selector("img[src]").unwrap().next();
    
    println!("{:?}", img);
}
```

Iterating over the subnodes of an HTML document:
```rust
fn main() {
    let input = r#"<div><img src="cool-image.png" /></div>"#;
    let dom = tl::parse(input, tl::ParserOptions::default());
    let img = dom.nodes()
        .iter()
        .find(|node| {
            node.as_tag().map_or(false, |tag| tag.name() == "img".into())
        });
    
    println!("{:?}", img);
}
```

## SIMD-accelerated parsing
This crate has optimized parsing functions which make use of SIMD. These are disabled by default and must be enabled explicitly by passing the `simd` feature flag due to the unstable feature `portable_simd`. This requires a **nightly** compiler!

## Benchmarks
Results for parsing a ~320KB [HTML document](https://github.com/y21/rust-html-parser-benchmark/blob/80d24a260ab9377bc704aa0b12657539aeaa4777/data/wikipedia.html).
Left and right numbers are lower/upper bounds of the confidence interval. The middle number is criterion's best estimate of time/throughput for each iteration.
```notrust
tl + simd
  time:   [627.03 us 628.23 us 629.48 us]
  thrpt:  [496.88 MiB/s 497.87 MiB/s 498.83 MiB/s]

html5ever
  time:   [5.7817 ms 5.7900 ms 5.7985 ms]
  thrpt:  [53.942 MiB/s 54.021 MiB/s 54.098 MiB/s]
  
htmlparser
  time:   [17.738 ms 17.764 ms 17.790 ms]
  thrpt:  [17.582 MiB/s 17.608 MiB/s 17.634 MiB/s]
  
rphtml
  time:   [6.0053 ms 6.0154 ms 6.0256 ms]
  thrpt:  [51.909 MiB/s 51.997 MiB/s 52.084 MiB/s]
  
rusthtml
  time:   [3.3830 ms 3.3881 ms 3.3933 ms]
  thrpt:  [92.175 MiB/s 92.317 MiB/s 92.455 MiB/s]
  
htmlstream
  time:   [2.2752 ms 2.2786 ms 2.2822 ms]
  thrpt:  [137.05 MiB/s 137.27 MiB/s 137.48 MiB/s]
```

[Source](https://github.com/y21/rust-html-parser-benchmark/tree/53238f68bbb57adc8dffdd245693ca1caa89cf4f) (file: wikipedia.html)

## Design
Due to the nature of zero-copy parsers, the string must be kept alive for the entire lifetime of the parser/dom.
If this is not acceptable or simply not possible in your case, you can call `tl::parse_owned()`.
This goes through the same steps as `tl::parse()` but returns an `VDomGuard` instead of a `VDom`.
The difference is that `VDomGuard` carefully creates a self-referential struct in which it stores the input string, so you can keep the `VDomGuard` as long as you want and move it around as much as you want.
