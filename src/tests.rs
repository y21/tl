use crate::{parse, parse_owned};
use crate::parser::*;

fn force_as_tag<'a, 'b>(actual: &'a Node<'b>) -> &'a HTMLTag<'b> {
    match actual {
        Node::Tag(t) => t,
        _ => panic!("Failed to force tag as Node::Tag (got {:?})", actual),
    }
}

#[test]
fn inner_html() {
    let dom = parse("abc <p>test</p> def");

    let tag = force_as_tag(&dom.children()[1]);

    assert_eq!(tag.inner_html().as_utf8_str(), "<p>test</p>");
}

#[test]
fn children_len() {
    let dom = parse("<!-- element 1 --> <div><div>element 3</div></div>");
    assert_eq!(dom.children().len(), 2);
}

#[test]
fn get_element_by_id() {
    let dom = parse("<div></div><p id=\"test\"></p><p></p>");

    let tag = dom.get_element_by_id("test").expect("Element not present");

    let el = force_as_tag(&tag);

    assert_eq!(el.inner_html().as_utf8_str(), "<p id=\"test\"></p>")
}

#[test]
fn html5() {
    let dom = parse("<!DOCTYPE html> hello");

    assert_eq!(dom.version(), Some(HTMLVersion::HTML5));
    assert_eq!(dom.children().len(), 1)
}

#[test]
fn nested_inner_text() {
    let dom = parse("<p>hello <p>nested element</p></p>");

    let el = force_as_tag(&dom.children()[0]);

    assert_eq!(el.inner_text(), "hello nested element");
}

#[test]
fn owned_dom() {
    let owned_dom = {
        let input = String::from("<p id=\"test\">hello</p>");
        let dom = unsafe { parse_owned(input) };
        dom
    };

    let dom = owned_dom.get_ref();

    let el = force_as_tag(&dom.children()[0]);

    assert_eq!(el.inner_text(), "hello");
}

#[test]
fn move_owned() {
    let input = String::from("<p id=\"test\">hello</p>");

    let guard = unsafe { parse_owned(input) };

    fn move_me<T>(p: T) -> T { p }

    let guard = std::thread::spawn(|| guard).join().unwrap();
    let guard = move_me(guard);

    let dom = guard.get_ref();

    let el = force_as_tag(&dom.children()[0]);

    assert_eq!(el.inner_text(), "hello");
}