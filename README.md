# tl
tl is a very fast, zero-copy HTML parser written in pure Rust. <br />

- [Usage](#usage)
- [Benchmarks](#benchmarks)
- [Design](#design)

## Usage
Add `tl` to your dependencies.
```toml
[dependencies]
tl = "0.4.1"
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

## Benchmarks
Results for parsing a ~320KB [HTML document](https://github.com/y21/rust-html-parser-benchmark/blob/80d24a260ab9377bc704aa0b12657539aeaa4777/data/wikipedia.html).
Left and right numbers are lower/upper bounds of the confidence interval. The middle number is criterion's best estimate of time/throughput for each iteration.
```notrust
tl
  time:   [705.44 us 706.45 us 707.47 us]
  thrpt:  [442.11 MiB/s 442.75 MiB/s 443.38 MiB/s]

html5ever
  time:   [5.7573 ms 5.7645 ms 5.7717 ms]
  thrpt:  [54.192 MiB/s 54.260 MiB/s 54.327 MiB/s]
  
htmlparser
  time:   [18.131 ms 18.155 ms 18.179 ms]
  thrpt:  [17.206 MiB/s 17.228 MiB/s 17.251 MiB/s]
  
rphtml
  time:   [6.0143 ms 6.0223 ms 6.0305 ms]
  thrpt:  [51.867 MiB/s 51.937 MiB/s 52.006 MiB/s]
  
rusthtml
  time:   [3.3389 ms 3.3433 ms 3.3477 ms]
  thrpt:  [93.433 MiB/s 93.556 MiB/s 93.676 MiB/s]
  
htmlstream
  time:   [2.0316 ms 2.0344 ms 2.0372 ms]
  thrpt:  [153.54 MiB/s 153.75 MiB/s 153.96 MiB/s]
```

[Source](https://github.com/y21/rust-html-parser-benchmark/tree/80d24a260ab9377bc704aa0b12657539aeaa4777) (file: wikipedia.html)

## Design
Due to the zero-copy nature of parsers, the string must be kept alive for the entire lifetime of the parser/dom.
If this is not acceptable or simply not possible in your case, you can call `tl::parse_owned()`.
This goes through the same steps as `tl::parse()` but returns an `OwnedVDom` instead of a `VDom`.
The difference is that `OwnedVDom` carefully creates a self-referential struct in which it stores the input string, so you can keep the `OwnedVDom` as long as you want and move it around as much as you want.