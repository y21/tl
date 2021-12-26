use crate::{parse, parse_owned};
use crate::{parser::*, HTMLTag, Node};

fn force_as_tag<'a, 'b>(actual: &'a Node<'b>) -> &'a HTMLTag<'b> {
    match actual {
        Node::Tag(t) => t,
        _ => panic!("Failed to force tag as Node::Tag (got {:?})", actual),
    }
}

#[test]
fn inner_html() {
    let dom = parse("abc <p>test</p> def", ParserOptions::default());
    let parser = dom.parser();

    let tag = force_as_tag(dom.children()[1].get(parser).unwrap());

    assert_eq!(tag.inner_html().as_utf8_str(), "<p>test</p>");
}

#[test]
fn children_len() {
    let dom = parse(
        "<!-- element 1 --> <div><div>element 3</div></div>",
        ParserOptions::default(),
    );
    assert_eq!(dom.children().len(), 2);
}

#[test]
fn get_element_by_id_default() {
    let dom = parse(
        "<div></div><p id=\"test\"></p><p></p>",
        ParserOptions::default(),
    );

    let tag = dom.get_element_by_id("test").expect("Element not present");

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_html().as_utf8_str(), "<p id=\"test\"></p>")
}

#[test]
fn get_element_by_id_tracking() {
    let dom = parse(
        "<div></div><p id=\"test\"></p><p></p>",
        ParserOptions::default().track_ids(),
    );

    let tag = dom.get_element_by_id("test").expect("Element not present");

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_html().as_utf8_str(), "<p id=\"test\"></p>")
}

#[test]
fn get_element_by_class_name_default() {
    let dom = parse(
        "<div></div><p class=\"a b\">hey</p><p></p>",
        ParserOptions::default(),
    );

    let tag = dom.get_elements_by_class_name("a").next().unwrap();

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_text(dom.parser()), "hey");
}

#[test]
fn get_element_by_class_name_tracking() {
    let dom = parse(
        "<div></div><p class=\"a b\">hey</p><p></p>",
        ParserOptions::default().track_ids(),
    );

    let tag = dom.get_elements_by_class_name("a").next().unwrap();

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_text(dom.parser()), "hey");
}

#[test]
fn html5() {
    let dom = parse("<!DOCTYPE html> hello", ParserOptions::default());

    assert_eq!(dom.version(), Some(HTMLVersion::HTML5));
    assert_eq!(dom.children().len(), 1)
}

#[test]
fn nested_inner_text() {
    let dom = parse(
        "<p>hello <p>nested element</p></p>",
        ParserOptions::default(),
    );
    let parser = dom.parser();

    let el = force_as_tag(dom.children()[0].get(parser).unwrap());

    assert_eq!(el.inner_text(parser), "hello nested element");
}

#[test]
fn owned_dom() {
    let owned_dom = {
        let input = String::from("<p id=\"test\">hello</p>");
        let dom = unsafe { parse_owned(input, ParserOptions::default()) };
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

    let guard = unsafe { parse_owned(input, ParserOptions::default()) };

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

    let dom = parse(input, ParserOptions::default());
    let parser = dom.parser();

    let tag = dom
        .nodes()
        .iter()
        .find(|x| x.as_tag().map_or(false, |x| x.name() == "span".into()));

    assert_eq!(
        tag.map(|tag| tag.inner_text(parser)),
        Some("whats up".into())
    )
}

#[test]
fn abrupt_attributes_stop() {
    let input = r#"<p "#;
    parse(input, ParserOptions::default());
}

#[test]
fn dom_nodes() {
    let input = r#"<p><p><a>nested</a></p></p>"#;
    let dom = parse(input, ParserOptions::default());
    let parser = dom.parser();
    let element = dom
        .nodes()
        .iter()
        .find(|x| x.as_tag().map_or(false, |x| x.name().eq(&"a".into())));

    assert_eq!(element.map(|x| x.inner_text(parser)), Some("nested".into()));
}

#[test]
fn fuzz() {
    // Some tests that would previously panic or end in an infinite loop
    // We don't need to assert anything here, just see that they finish
    parse("J\x00<", ParserOptions::default());
    parse("<!J", ParserOptions::default());

    // Very deeply nested tags should not trigger a stack overflow
    parse(&"<p>".repeat(10000), ParserOptions::default());
}

#[cfg(feature = "simd")]
mod simd {
    use crate::util;

    #[test]
    fn string_search() {
        assert_eq!(util::find_fast(b"a", b' '), None);
        assert_eq!(util::find_fast(b"", b' '), None);
        assert_eq!(util::find_fast(b"a ", b' '), Some(1));
        assert_eq!(util::find_fast(b"abcd ", b' '), Some(4));
        assert_eq!(util::find_fast(b"ab cd ", b' '), Some(2));
        assert_eq!(util::find_fast(b"abcdefgh ", b' '), Some(8));
        assert_eq!(util::find_fast(b"abcdefghi ", b' '), Some(9));
        assert_eq!(util::find_fast(b"abcdefghi", b' '), None);
        assert_eq!(util::find_fast(b"abcdefghiabcdefghi .", b' '), Some(18));
        assert_eq!(util::find_fast(b"abcdefghiabcdefghi.", b' '), None);

        let long = "a".repeat(100000) + "b";
        assert_eq!(util::find_fast(long.as_bytes(), b'b'), Some(100000));
    }
}

#[test]
fn query_selector_simple() {
    let input = "<div><p class=\"hi\">hello</p></div>";
    let dom = parse(input, ParserOptions::default());
    let parser = dom.parser();
    let mut selector = dom.query_selector(".hi").unwrap();
    let el = force_as_tag(selector.next().and_then(|x| x.get(parser)).unwrap());

    assert_eq!(dom.nodes().len(), 3);
    assert_eq!(el.inner_text(parser), "hello");
}

#[test]
fn valueless_attribute() {
    // https://github.com/y21/tl/issues/11
    let input = r#"
        <a id="u54423">
            <iframe allowfullscreen></iframe>
        </a>
    "#;

    let dom = parse(input, ParserOptions::default());
    let element = dom.get_element_by_id("u54423");

    assert!(element.is_some());
}

#[test]
fn unquoted() {
    // https://github.com/y21/tl/issues/12
    let input = r#"
        <a id=u54423>Hello World</a>
    "#;

    let dom = parse(input, ParserOptions::default());
    let parser = dom.parser();
    let element = dom.get_element_by_id("u54423");

    assert_eq!(
        element.and_then(|x| x.get(parser).map(|x| x.inner_text(parser))),
        Some("Hello World".into())
    );
}
