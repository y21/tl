// For tests, we just import everything
use crate::*;

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
    assert_eq!(dom.children().len(), 3);
}

#[test]
fn get_element_by_id() {
    let dom = parse("<div></div><p id=\"test\"></p><p></p>");

    let tag = dom.get_element_by_id("test").expect("Element not present");

    let el = force_as_tag(&*tag);

    assert_eq!(el.inner_html().as_utf8_str(), "<p id=\"test\"></p>")
}
