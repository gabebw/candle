use scraper::{ElementRef, Node};

const NUMBER_OF_SPACES_PER_LEVEL: usize = 2;
// https://developer.mozilla.org/en-US/docs/Glossary/empty_element
const SELF_CLOSING_ELEMENTS: &[&str; 16] = &[
    "area",
    "base",
    "br",
    "col",
    "command",
    "embed",
    "hr",
    "img",
    "input",
    "keygen",
    "link",
    "meta",
    "param",
    "source",
    "track",
    "wbr"
];

fn is_present(s: &str) -> bool {
    s.chars().any(|c| !c.is_whitespace())
}

fn indentation(level: usize) -> String {
    format!("{:n$}", "", n=level * NUMBER_OF_SPACES_PER_LEVEL)
}

fn add_children(s: &mut String, element: ElementRef, indent_level: usize) {
    let indent_plus_one = indentation(indent_level + 1);

    for child in element.children() {
        // Each `child` is a NodeRef<Node>:
        // https://docs.rs/ego-tree/0.3.0/ego_tree/struct.NodeRef.html
        // https://docs.rs/scraper/0.10.1/scraper/node/enum.Node.html
        match child.value() {
            Node::Comment(c) => {
                s.push_str(&format!("\n{}<!-- {} -->", indent_plus_one, c.comment.trim()))
            },
            Node::Element(_) => {
                if let Some(element) = ElementRef::wrap(child) {
                    s.push_str("\n");
                    s.push_str(&print_tree(element, indent_level + 1));
                }
            },
            Node::Text(t) => {
                if is_present(&t.text) {
                    s.push_str(&format!("\n{}{}", indent_plus_one, t.text))
                }
            },
            _ => {},
        }
    }
}

pub fn print_tree(element: ElementRef, indent_level: usize) -> String {
    let mut s = String::new();
    let indent = indentation(indent_level);
    // `element` is https://docs.rs/scraper/0.10.1/scraper/element_ref/struct.ElementRef.html
    let top = element.value();
    let tag_name: &str = &format!("{}", top.name.local);

    if SELF_CLOSING_ELEMENTS.contains(&tag_name) {
        // The tag is self-closing and can't have any children
        s.push_str(&format!("{}{:?}</{}>", indent, top, tag_name));
    } else {
        // Opening tag, with attributes
        s.push_str(&format!("{}{:?}", indent, top));
        add_children(&mut s, element, indent_level);
        // Closing tag
        s.push_str(&format!("\n{}</{}>", indent, tag_name));
    }
    s
}
