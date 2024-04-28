use crate::{parse, parse_owned, Bytes, VDom};
use crate::{parser::*, HTMLTag, Node};

fn force_as_tag<'a, 'b>(actual: &'a Node<'b>) -> &'a HTMLTag<'b> {
    match actual {
        Node::Tag(t) => t,
        _ => panic!("Failed to force tag as Node::Tag (got {:?})", actual),
    }
}

#[test]
fn outer_html() {
    let dom = parse(
        "abc <p>test<span>a</span></p> def",
        ParserOptions::default(),
    )
    .unwrap();
    let parser = dom.parser();

    let tag = force_as_tag(dom.children()[1].get(parser).unwrap());

    assert_eq!(tag.outer_html(parser), "<p>test<span>a</span></p>");
}

#[test]
fn outer_html_void_elements() {
    const HTML_INPUT: &str = r#"<html><head></head><body><img src=""><br><hr></body></html>"#;
    let vdom = parse(HTML_INPUT, ParserOptions::default()).unwrap();
    assert_eq!(
        r#"<html><head></head><body><img src=""><br><hr></body></html>"#,
        vdom.outer_html()
    );
}

#[test]
fn inner_html() {
    let dom = parse(
        "abc <p>test<span>a</span></p> def",
        ParserOptions::default(),
    )
    .unwrap();
    let parser = dom.parser();

    let tag = force_as_tag(dom.children()[1].get(parser).unwrap());

    assert_eq!(tag.inner_html(parser), "test<span>a</span>");
}

#[test]
fn children_len() {
    let dom = parse(
        "<!-- element 1 --><div><div>element 3</div></div>",
        ParserOptions::default(),
    )
    .unwrap();
    assert_eq!(dom.children().len(), 2);
}

#[test]
fn get_element_by_id_default() {
    let dom = parse(
        "<div></div><p id=\"test\"></p><p></p>",
        ParserOptions::default(),
    )
    .unwrap();
    let parser = dom.parser();

    let tag = dom.get_element_by_id("test").expect("Element not present");

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.outer_html(parser), "<p id=\"test\"></p>")
}

#[test]
fn get_element_by_id_tracking() {
    let dom = parse(
        "<div></div><p id=\"test\"></p><p></p>",
        ParserOptions::default().track_ids(),
    )
    .unwrap();
    let parser = dom.parser();

    let tag = dom.get_element_by_id("test").expect("Element not present");

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.outer_html(parser), "<p id=\"test\"></p>")
}

#[test]
fn get_element_by_class_name_default() {
    let dom = parse(
        "<div></div><p class=\"a b\">hey</p><p></p>",
        ParserOptions::default(),
    )
    .unwrap();

    let tag = dom.get_elements_by_class_name("a").next().unwrap();

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_text(dom.parser()), "hey");
}

#[test]
fn get_element_by_class_name_tracking() {
    let dom = parse(
        "<div></div><p class=\"a b\">hey</p><p></p>",
        ParserOptions::default().track_ids(),
    )
    .unwrap();

    let tag = dom.get_elements_by_class_name("a").next().unwrap();

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_text(dom.parser()), "hey");
}

#[test]
fn html5() {
    let dom = parse("<!DOCTYPE html> hello", ParserOptions::default()).unwrap();

    assert_eq!(dom.version(), Some(HTMLVersion::HTML5));
    assert_eq!(dom.children().len(), 1)
}

#[test]
fn ignore_void_closing_tags() {
    let input = r#"
        <head>
            <base href='single_quoted_item'></base>
            <link rel="stylesheet" type="text/css" href="non-exising"/>
        </head>
    "#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let head_tag = force_as_tag(dom.children()[1].get(dom.parser()).unwrap());

    let base_tag = force_as_tag(head_tag.children().top()[1].get(dom.parser()).unwrap());
    let link_tag = force_as_tag(head_tag.children().top()[3].get(dom.parser()).unwrap());

    assert_eq!(head_tag.name(), "head");
    assert_eq!(base_tag.name(), "base");
    assert_eq!(link_tag.name(), "link");
}

#[test]
pub fn children_mut() {
    let input = "<head><p>Replace me</p> World</head>";

    let mut dom = parse(input, Default::default()).unwrap();
    let children = dom.children();
    let child = children[0]
        .clone()
        .get_mut(dom.parser_mut())
        .unwrap()
        .as_tag_mut()
        .unwrap();

    let mut children = child.children_mut();
    let top = children.top_mut();
    let handle = top[0].clone();
    let node = handle.get_mut(dom.parser_mut()).unwrap();
    *node = Node::Raw("Hello".into());

    assert_eq!(dom.outer_html(), "<head>Hello World</head>");
}

#[test]
fn nested_inner_text() {
    let dom = parse(
        "<p>hello <p>nested element</p></p>",
        ParserOptions::default(),
    )
    .unwrap();
    let parser = dom.parser();

    let el = force_as_tag(dom.children()[0].get(parser).unwrap());

    assert_eq!(el.inner_text(parser), "hello nested element");
}

#[test]
fn owned_dom() {
    let owned_dom = {
        let input = String::from("<p id=\"test\">hello</p>");
        let dom = unsafe { parse_owned(input, ParserOptions::default()).unwrap() };
        dom
    };

    let dom = owned_dom.get_ref();
    let parser = dom.parser();

    let el = force_as_tag(dom.children()[0].get(parser).unwrap());

    assert_eq!(el.inner_text(parser), "hello");
}

#[test]
fn move_owned() {
    let input = String::from("<p id=\"test\">hello</p>");

    let guard = unsafe { parse_owned(input, ParserOptions::default()).unwrap() };

    fn move_me<T>(p: T) -> T {
        p
    }

    let guard = std::thread::spawn(|| guard).join().unwrap();
    let guard = move_me(guard);

    let dom = guard.get_ref();
    let parser = dom.parser();

    let el = force_as_tag(dom.children()[0].get(parser).unwrap());

    assert_eq!(el.inner_text(parser), "hello");
}

#[test]
fn with() {
    let input = r#"<p>hello <span>whats up</span></p>"#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();

    let tag = dom
        .nodes()
        .iter()
        .find(|x| x.as_tag().map_or(false, |x| x.name() == "span"));

    assert_eq!(
        tag.map(|tag| tag.inner_text(parser)),
        Some("whats up".into())
    )
}

#[test]
fn abrupt_attributes_stop() {
    let input = r#"<p "#;
    parse(input, ParserOptions::default()).unwrap();
}

#[test]
fn dom_nodes() {
    let input = r#"<p><p><a>nested</a></p></p>"#;
    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let element = dom
        .nodes()
        .iter()
        .find(|x| x.as_tag().map_or(false, |x| x.name().eq("a")));

    assert_eq!(element.map(|x| x.inner_text(parser)), Some("nested".into()));
}

#[test]
fn fuzz() {
    // Some tests that would previously panic or end in an infinite loop
    // We don't need to assert anything here, just see that they finish
    parse("J\x00<", ParserOptions::default()).unwrap();
    parse("<!J", ParserOptions::default()).unwrap();
    parse("<=/Fy<=/", Default::default()).unwrap();

    // Miri is too slow... :(
    let count = if cfg!(miri) { 100usize } else { 10000usize };

    parse(&"<p>".repeat(count), ParserOptions::default()).unwrap();
}

#[test]
fn mutate_dom() {
    let input = r#"<img src="test.png" />"#;
    let mut dom = parse(input, ParserOptions::default()).unwrap();

    let mut selector = dom.query_selector("[src]").unwrap();
    let handle = selector.next().unwrap();

    let parser = dom.parser_mut();

    let el = handle.get_mut(parser).unwrap();
    let tag = el.as_tag_mut().unwrap();
    let attr = tag.attributes_mut();
    let bytes = attr.get_mut("src").flatten().unwrap();
    bytes.set("world.png").unwrap();

    assert_eq!(attr.get("src"), Some(Some(&"world.png".into())));
}

mod simd {
    // These tests make sure that SIMD functions do the right thing

    #[test]
    fn matches_case_insensitive_test() {
        assert!(crate::simd::matches_case_insensitive(b"hTmL", *b"html"));
        assert!(!crate::simd::matches_case_insensitive(b"hTmLs", *b"html"));
        assert!(!crate::simd::matches_case_insensitive(b"hTmy", *b"html"));
        assert!(!crate::simd::matches_case_insensitive(b"/Tmy", *b"html"));
    }

    #[test]
    fn string_search() {
        assert_eq!(crate::simd::find(b"a", b' '), None);
        assert_eq!(crate::simd::find(b"", b' '), None);
        assert_eq!(crate::simd::find(b"a ", b' '), Some(1));
        assert_eq!(crate::simd::find(b"abcd ", b' '), Some(4));
        assert_eq!(crate::simd::find(b"ab cd ", b' '), Some(2));
        assert_eq!(crate::simd::find(b"abcdefgh ", b' '), Some(8));
        assert_eq!(crate::simd::find(b"abcdefghi ", b' '), Some(9));
        assert_eq!(crate::simd::find(b"abcdefghi", b' '), None);
        assert_eq!(crate::simd::find(b"abcdefghiabcdefghi .", b' '), Some(18));
        assert_eq!(crate::simd::find(b"abcdefghiabcdefghi.", b' '), None);

        let count = if cfg!(miri) { 500usize } else { 1000usize };

        let long = "a".repeat(count) + "b";
        assert_eq!(crate::simd::find(long.as_bytes(), b'b'), Some(count));
    }

    #[test]
    fn string_search_4() {
        const NEEDLE: [u8; 4] = [b'a', b'b', b'c', b'd'];

        assert_eq!(crate::simd::find4(b"e", NEEDLE), None);
        assert_eq!(crate::simd::find4(b"a", NEEDLE), Some(0));
        assert_eq!(crate::simd::find4(b"ea", NEEDLE), Some(1));
        assert_eq!(crate::simd::find4(b"ef", NEEDLE), None);
        assert_eq!(crate::simd::find4(b"ef a", NEEDLE), Some(3));
        assert_eq!(crate::simd::find4(b"ef g", NEEDLE), None);
        assert_eq!(crate::simd::find4(b"ef ghijk", NEEDLE), None);
        assert_eq!(crate::simd::find4(b"ef ghijkl", NEEDLE), None);
        assert_eq!(crate::simd::find4(b"ef ghijkla", NEEDLE), Some(9));
        assert_eq!(crate::simd::find4(b"ef ghiajklm", NEEDLE), Some(6));
        assert_eq!(crate::simd::find4(b"ef ghibjklm", NEEDLE), Some(6));
        assert_eq!(crate::simd::find4(b"ef ghicjklm", NEEDLE), Some(6));
        assert_eq!(crate::simd::find4(b"ef ghidjklm", NEEDLE), Some(6));
        assert_eq!(crate::simd::find4(b"ef ghijklmnopqrstua", NEEDLE), Some(18));
        assert_eq!(crate::simd::find4(b"ef ghijklmnopqrstub", NEEDLE), Some(18));
        assert_eq!(crate::simd::find4(b"ef ghijklmnopqrstuc", NEEDLE), Some(18));
        assert_eq!(crate::simd::find4(b"ef ghijklmnopqrstud", NEEDLE), Some(18));
        assert_eq!(crate::simd::find4(b"ef ghijklmnopqrstu", NEEDLE), None);
    }

    #[test]
    #[rustfmt::skip]
    fn search_non_ident() {
        assert_eq!(crate::simd::search_non_ident(b"this-is-a-very-long-identifier<"), Some(30));
        assert_eq!(crate::simd::search_non_ident(b"0123456789Abc_-<"), Some(15));
        assert_eq!(crate::simd::search_non_ident(b"0123456789Abc-<"), Some(14));
        assert_eq!(crate::simd::search_non_ident(b"0123456789Abcdef_-<"), Some(18));
        assert_eq!(crate::simd::search_non_ident(b""), None);
        assert_eq!(crate::simd::search_non_ident(b"short"), None);
        assert_eq!(crate::simd::search_non_ident(b"short_<"), Some(6));
        assert_eq!(crate::simd::search_non_ident(b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_"), None);
        assert_eq!(crate::simd::search_non_ident(b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_<"), Some(64));
        assert_eq!(crate::simd::search_non_ident(b"0123456789ab<defghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_<"), Some(12));
    }
}

mod bytes {
    use crate::bytes::*;

    #[test]
    fn from_str() {
        let x = Bytes::from("hello");
        assert_eq!(x.as_bytes(), b"hello");
    }

    #[test]
    fn from_bytes() {
        let x = Bytes::from(b"hello" as &[u8]);
        assert_eq!(x.as_bytes(), b"hello");
    }

    #[test]
    fn as_bytes_borrowed() {
        let xb = Bytes::from(b"hello" as &[u8]);
        assert_eq!(xb.as_bytes_borrowed(), Some(b"hello" as &[u8]));

        let mut xc = xb.clone();
        xc.set("test2").unwrap();
        assert_eq!(xc.as_bytes_borrowed(), None);
    }

    #[test]
    fn as_utf8_str() {
        assert_eq!(Bytes::from("hello").as_utf8_str(), "hello");
    }

    #[test]
    fn clone_shallow() {
        // cloning a borrowed slice does not deep-clone
        let x = Bytes::from("hello");
        let xp = x.as_ptr();

        let y = x.clone();
        let yp = y.as_ptr();

        assert_eq!(xp, yp);
    }

    #[test]
    fn drop_old_owned() {
        let mut x = Bytes::from("");
        x.set("test").unwrap();
        x.set("test2").unwrap();
    }

    #[test]
    fn clone_owned_deep() {
        let mut x = Bytes::from("");
        x.set("hello").unwrap();
        let xp = x.as_ptr();

        let y = x.clone();
        let yp = y.as_ptr();

        assert_eq!(x, y);
        assert_ne!(xp, yp);
    }

    #[test]
    fn empty() {
        let _x = Bytes::new();
    }

    #[test]
    fn empty_set() {
        let mut x = Bytes::new();
        x.set("hello").unwrap();
    }

    #[test]
    fn set() {
        let mut x = Bytes::from("hello");
        let xp = x.as_ptr();

        x.set("world").unwrap();
        let xp2 = x.as_ptr();

        // check that the changes are reflected
        assert_eq!(x.as_bytes(), b"world");

        // pointer must be different now as the call to `set` should cause an allocation
        assert_ne!(xp, xp2);
    }

    #[test]
    fn clone_deep() {
        let x = Bytes::from("hello");
        let xp = x.as_ptr();

        let mut y = x.clone();
        y.set("world").unwrap();
        let yp = y.as_ptr();

        assert_ne!(xp, yp);
    }

    #[test]
    fn into_owned_bytes() {
        let mut x1 = Bytes::new();
        x1.set("hello").unwrap(); // &str

        let mut x2 = x1.clone();
        x2.set(b"world" as &[u8]).unwrap(); // &[u8]

        let mut x3 = x1.clone();
        x3.set(vec![0u8, 1, 2, 3, 4]).unwrap(); // Vec<u8>

        let mut x4 = x1.clone();
        x4.set(vec![0u8, 1, 2, 3, 4].into_boxed_slice()).unwrap(); // Box<[u8]>

        let mut x5 = x1.clone();
        x5.set(String::from("Tests are important")).unwrap(); // String
    }
}

#[test]
fn valueless_attribute() {
    // https://github.com/y21/tl/issues/11
    let input = r#"
        <a id="u54423">
            <iframe allowfullscreen></iframe>
        </a>
    "#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let element = dom.get_element_by_id("u54423");

    assert!(element.is_some());
}

#[test]
fn unquoted() {
    // https://github.com/y21/tl/issues/12
    let input = r#"
        <a id=u54423>Hello World</a>
    "#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let element = dom.get_element_by_id("u54423");

    assert_eq!(
        element.and_then(|x| x.get(parser).map(|x| x.inner_text(parser))),
        Some("Hello World".into())
    );
}

mod query_selector {
    use super::*;
    #[test]
    fn query_selector_simple() {
        let input = "<div><p class=\"hi\">hello</p></div>";
        let dom = parse(input, ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let mut selector = dom.query_selector(".hi").unwrap();
        let el = force_as_tag(selector.next().and_then(|x| x.get(parser)).unwrap());

        assert_eq!(dom.nodes().len(), 3);
        assert_eq!(el.inner_text(parser), "hello");
    }

    #[test]
    fn tag_query_selector() {
        // empty
        let dom = parse("<p></p>", ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let selector = dom.nodes()[0]
            .as_tag()
            .unwrap()
            .query_selector(parser, "div.z")
            .unwrap();
        assert_eq!(selector.count(), 0);

        // one child
        let dom = parse(
            r#"<p><div class="z">PASS</div></p>"#,
            ParserOptions::default(),
        )
        .unwrap();
        let parser = dom.parser();
        let mut selector = dom.nodes()[0]
            .as_tag()
            .unwrap()
            .query_selector(parser, "div.z")
            .unwrap();
        assert_eq!(selector.clone().count(), 1);
        assert_eq!(
            selector
                .next()
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser),
            "PASS"
        );

        // nested
        let dom = parse(
            r#"<p><div class="z"><div class="y">PASS</div></div></p>"#,
            ParserOptions::default(),
        )
        .unwrap();
        let parser = dom.parser();
        let mut selector = dom.nodes()[0]
            .as_tag()
            .unwrap()
            .query_selector(parser, "div.y")
            .unwrap();
        assert_eq!(selector.clone().count(), 1);
        assert_eq!(
            selector
                .next()
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser),
            "PASS"
        );
    }

    #[test]
    fn query_selector_with_quote() {
        let input = r#"<div><meta property="og:title" content="hello" /></div>"#;
        let dom = parse(input, ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let node_option = dom
            .query_selector(r#"meta[property="og:title"]"#)
            .and_then(|mut iter| iter.next());
        let value = if let Some(node) = node_option {
            Some(
                node.get(parser)
                    .unwrap()
                    .as_tag()
                    .unwrap()
                    .attributes()
                    .get("content")
                    .flatten()
                    .unwrap()
                    .try_as_utf8_str()
                    .unwrap()
                    .to_string(),
            )
        } else {
            None
        };

        assert_eq!(value, Some("hello".to_string()));
    }
}

#[test]
fn nodes_order() {
    let input = r#"
    <p>test</p><div><span>test2</span></div>
    "#
    .trim();
    let dom = parse(input, Default::default()).unwrap();
    let nodes = dom.nodes();

    // 5 nodes in total
    assert_eq!(nodes.len(), 5);

    // First node is <p>
    assert_eq!(&nodes[0].as_tag().unwrap()._name, "p");
    // Second node is inner text of <p>: test
    assert_eq!(nodes[1].as_raw().unwrap().as_bytes(), b"test");
    // Third node is <div>
    assert_eq!(&nodes[2].as_tag().unwrap()._name, "div");
    // Fourth node is inner <span> node
    assert_eq!(&nodes[3].as_tag().unwrap()._name, "span");
    // Fifth node is inner text of <span>: test2
    assert_eq!(nodes[4].as_raw().unwrap().as_bytes(), b"test2");
}

#[test]
fn comment() {
    let dom = parse("<!-- test -->", Default::default()).unwrap();
    let nodes = dom.nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(
        nodes[0].as_comment().unwrap().as_utf8_str(),
        "<!-- test -->"
    );
}

#[test]
fn tag_all_children() {
    fn assert_len(input: &str, len: usize) {
        let dom = parse(input, Default::default()).unwrap();
        let el = dom.nodes()[0].as_tag().unwrap();
        assert_eq!(el.children().all(dom.parser()).len(), len);
    }

    fn assert_last(input: &str, last: &str) {
        let dom = parse(input, Default::default()).unwrap();
        let el = dom.nodes()[0].as_tag().unwrap();
        assert_eq!(
            el.children()
                .all(dom.parser())
                .last()
                .unwrap()
                .inner_text(dom.parser()),
            last
        );
    }

    assert_len(r#"<div></div>"#, 0);
    assert_len(r#"<div>a</div>"#, 1);
    assert_len(r#"<div><p></p></div>"#, 1);
    assert_len(r#"<div><p>a</p></div>"#, 2);
    assert_len(r#"<div><p><span></span></p></div>"#, 2);
    assert_len(r#"<div><p><span>a</span></p></div>"#, 3);

    assert_last(r#"<div>a</div>"#, "a");
    assert_last(r#"<div><p>a</p></div>"#, "a");
    assert_last(r#"<div>b<p>a</p></div>"#, "a");
    assert_last(r#"<div>b<p><span>a</span></p></div>"#, "a");
}

#[test]
fn assert_length() {
    fn assert_len(input: &str, selector: &str, len: usize) {
        let dom = parse(input, Default::default()).unwrap();
        let el = dom.nodes()[0].as_tag().unwrap();
        let query = el.query_selector(dom.parser(), selector).unwrap();
        assert_eq!(query.count(), len);
    }

    assert_len("<div></div>", "a", 0);
    assert_len("<div><a></a></div>", "a", 1);
    assert_len("<div><a><a></a></a></div>", "a", 2);
    assert_len("<div><a><span></span></a></div>", "span", 1);
}

#[test]
fn self_closing_no_child() {
    let dom = parse("<br /><p>test</p>", Default::default()).unwrap();
    let nodes = dom.nodes();
    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0].as_tag().unwrap()._children.len(), 0);
    assert_eq!(nodes[0].as_tag().unwrap().raw(), "<br />");
}

#[test]
fn insert_attribute_owned() {
    // https://github.com/y21/tl/issues/27
    let mut attr = Attributes::new();
    let style = "some style".to_string();
    attr.insert("style", Some(Bytes::try_from(style).unwrap()));
    assert_eq!(attr.get("style"), Some(Some(&"some style".into())));
}

#[test]
fn boundaries() {
    // https://github.com/y21/tl/issues/25
    let dom = parse("<div><p>haha</p></div>", Default::default()).unwrap();
    let span = dom.nodes()[1].as_tag().unwrap();
    let boundary = span.boundaries(dom.parser());
    assert_eq!(boundary, (5, 15));
}

#[test]
fn attributes_remove_inner_html() {
    let mut dom = parse(
        "<span contenteditable=\"true\">testing</a>",
        Default::default(),
    )
    .unwrap();

    dom.nodes_mut()[0]
        .as_tag_mut()
        .unwrap()
        .attributes_mut()
        .remove_value("contenteditable");

    assert_eq!(dom.outer_html(), "<span contenteditable>testing</span>");

    dom.nodes_mut()[0]
        .as_tag_mut()
        .unwrap()
        .attributes_mut()
        .remove("contenteditable");

    assert_eq!(dom.outer_html(), "<span>testing</span>");
}

#[test]
fn tag_raw() {
    let input = "<p>abcd</p>";

    let vdom = parse(input, Default::default()).unwrap();
    let first_tag = vdom.children()[0]
        .get(vdom.parser())
        .unwrap()
        .as_tag()
        .unwrap();

    let from_raw = first_tag.raw().try_as_utf8_str().unwrap();
    assert_eq!(from_raw, "<p>abcd</p>");
}

#[test]
fn tag_raw_abrupt_stop() {
    let input = "<p>abcd</p";

    let vdom = parse(input, Default::default()).unwrap();
    let first_tag = vdom.children()[0]
        .get(vdom.parser())
        .unwrap()
        .as_tag()
        .unwrap();

    let from_raw = first_tag.raw().try_as_utf8_str().unwrap();
    assert_eq!(from_raw, "<p>abcd</p");
}

#[test]
fn line_breaks_inside_tags() {
    /// Asserts that the given node is a div with the given text as its only child.
    fn assert_div_with_text(node: &NodeHandle, dom: &VDom, text: &[u8]) {
        let tag = node.get(dom.parser()).unwrap().as_tag().unwrap();
        assert_eq!(&tag._name, "div");
        assert_eq!(tag._children.len(), 1);
        assert_eq!(
            tag._children[0]
                .get(dom.parser())
                .unwrap()
                .as_raw()
                .unwrap()
                .as_bytes(),
            text
        );
    }

    let input = "<div>div 1</div><\ndiv\n>div 2</\ndiv\n><div>div 3</div><div>div 4</div>";
    let dom = parse(input, Default::default()).unwrap();
    let children = dom.children();

    // each of the 4 divs should be a top level child
    assert_eq!(children.len(), 4);

    assert_div_with_text(&children[0], &dom, b"div 1");
    assert_div_with_text(&children[1], &dom, b"div 2");
    assert_div_with_text(&children[2], &dom, b"div 3");
    assert_div_with_text(&children[3], &dom, b"div 4");
}
