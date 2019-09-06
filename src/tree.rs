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

fn presence(s: &str) -> Option<&str> {
    if is_present(s) {
        Some(s)
    } else {
        None
    }
}

// Given a line with N spaces of indentation, return N.
fn find_indentation(s: &str) -> usize {
    s.chars().take_while(|c| c.is_ascii_whitespace()).count()
}

fn indentation(level: usize) -> String {
    format!("{:n$}", "", n=level * NUMBER_OF_SPACES_PER_LEVEL)
}

fn add_children(s: &mut String, element: ElementRef, tag_name: &str, indent_level: usize) {
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
                    if tag_name == "script" {
                        s.push_str("\n");
                        s.push_str(&indented_script_text(&t.text, &indent_plus_one));
                    } else {
                        s.push_str(&format!("\n{}{}", indent_plus_one, t.text));
                    }
                }
            },
            _ => {},
        }
    }
}

// Trim up to N characters from the start. Will stop trimming if it runs into non-whitespace, or
// once it's trimmed N characters.
pub fn trim_start_n(s: &str, n: usize) -> String {
    let mut i = 0;
    let mut new = String::new();
    let mut stop_trimming = false;
    for c in s.chars() {
        if c.is_whitespace() && i < n && !stop_trimming {
            // keep going
            i += 1;
        } else {
            stop_trimming = true;
            new.push(c);
        }
    }
    new
}

fn indented_script_text(s: &str, indent: &str) -> String {
    let lines: Vec<_> = s.trim_end().lines().filter_map(presence).collect();
    let script_indentation_spaces = find_indentation(lines[0]);
    let indented: Vec<String> = lines
        .iter()
        .map(|l| format!("{}{}", indent, trim_start_n(l, script_indentation_spaces)))
        .collect();
    indented.join("\n")
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
        add_children(&mut s, element, tag_name, indent_level);
        // Closing tag
        s.push_str(&format!("\n{}</{}>", indent, tag_name));
    }
    s
}
