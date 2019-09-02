use encoding_rs::Encoding;
use isatty::stdin_isatty;
use regex::Regex;
use scraper::{ElementRef, Html, Node, Selector};
use std::cmp;
use std::env;
use std::io::{self, Read, Write};
use std::process;

struct Inputs {
    selector: String,
    html: String,
}

#[derive(Debug)]
enum FinderOperation<'a> {
    Attr(&'a str),
    Text,
    Html,
}

#[derive(Debug)]
struct Finder<'a> {
    selector: Selector,
    operation: FinderOperation<'a>,
}

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

fn print_tree(element: Option<ElementRef>, indent_level: usize) -> String {
    let mut s = String::new();
    let indent = format!("{:n$}", "", n=(indent_level * NUMBER_OF_SPACES_PER_LEVEL));
    let indent_plus_one = format!("{:n$}", "", n=(indent_level + 1) * NUMBER_OF_SPACES_PER_LEVEL);
    if let Some(element) = element {
        // `element` is https://docs.rs/scraper/0.10.1/scraper/element_ref/struct.ElementRef.html
        let top = element.value();
        let tag_name: &str = &format!("{}", top.name.local);

        if SELF_CLOSING_ELEMENTS.contains(&tag_name) {
            // The tag is self-closing and can't have any children
            s.push_str(&format!("{}{:?}</{}>", indent, top, tag_name));
        } else {
            // Opening tag, with attributes
            s.push_str(&format!("{}{:?}", indent, top));

            for child in element.children() {
                // Each `child` is a NodeRef<Node>:
                // https://docs.rs/ego-tree/0.3.0/ego_tree/struct.NodeRef.html
                // https://docs.rs/scraper/0.10.1/scraper/node/enum.Node.html
                match child.value() {
                    Node::Comment(c) => {
                        s.push_str(&format!("\n{}<!-- {} -->", indent_plus_one, c.comment.trim()))
                    },
                    Node::Element(_) => {
                        s.push_str("\n");
                        s.push_str(&print_tree(ElementRef::wrap(child), indent_level + 1));
                    },
                    Node::Text(t) => {
                        // Ignore this text if it's just whitespace
                        if ! t.text.trim().is_empty() {
                            s.push_str(&format!("\n{}{}", indent_plus_one, t.text))
                        }
                    },
                    _ => {},
                }
            }

            // Closing tag
            s.push_str(&format!("\n{}</{}>", indent, tag_name));
        }
    }
    s
}

impl<'a> Finder<'a> {
    fn apply(&self, element: &ElementRef) -> Option<String> {
        match self.operation {
            FinderOperation::Text => Some(element.text().collect()),
            FinderOperation::Attr(attr) => element.value().attr(attr).map(|s| s.to_string()),
            FinderOperation::Html => Some(print_tree(Some(*element), 0))
        }
    }
}

fn read_from<R: Read>(mut reader: R) -> Option<String> {
    // It might not be valid UTF-8, so read to a vector of bytes and convert it to UTF-8, lossily
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_to_end(&mut buffer).ok()?;
    let re_meta_charset = Regex::new(r#"<meta\s+charset=["']([^'"]+)["']"#).unwrap();
    let string = String::from_utf8_lossy(&buffer).to_string();
    let len = cmp::min(string.len(), 1024);
    if let Some(captures) = re_meta_charset.captures(&string[..len]) {
        let charset = captures.get(1).unwrap().as_str();
        match Encoding::for_label(charset.as_bytes()) {
            Some(encoding) => {
                let string_with_new_encoding = encoding.decode(&buffer).0;
                Some((*string_with_new_encoding).to_string())
            }
            None => Some(string),
        }
    } else {
        Some(string)
    }
}

fn read_inputs() -> Result<Inputs, String> {
    let selector = env::args().nth(1).ok_or("Usage: candle SELECTOR")?;
    let html = read_from(io::stdin()).ok_or("Error: couldn't read from STDIN")?;
    Ok(Inputs { selector, html })
}

fn main() {
    if stdin_isatty() {
        eprintln!("You must pipe in input to candle");
        process::exit(1);
    }

    match read_inputs() {
        Ok(inputs) => match parse(inputs) {
            Ok(result) => {
                let mut stdout = io::stdout();

                for r in result {
                    // Ignore the Err because the most common Err is exactly the one to suppress:
                    // panicking when piping to something that truncates the output, like `head`.
                    if writeln!(stdout, "{}", &r.trim()).is_err() {
                        break;
                    }
                }
            },
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn select_all(html: Html, finders: &[Finder]) -> Vec<String> {
    let mut results: Vec<String> = Vec::new();
    for node in html.tree.nodes().by_ref() {
        if let Some(element) = ElementRef::wrap(node) {
            if element.parent().is_some() {
                for finder in finders {
                    if finder.selector.matches(&element) {
                        if let Some(value) = finder.apply(&element) {
                            results.push(value);
                        }
                    }
                }
            }
        }
    }
    results
}

fn parse(inputs: Inputs) -> Result<Vec<String>, String> {
    let re = Regex::new(r"(?x)
        (?P<selector>[^{}]+)
        (?:
            (?P<text>\{text\})
            |
            (?P<html>\{html\})
            |
            (attr\{
                (?P<attr>[^}]+)
            \})
        )
        [,]?\s*
    ").unwrap();
    let mut finders: Vec<Finder> = Vec::new();
    for c in re.captures_iter(&inputs.selector) {
        let selector_str = c.name("selector").unwrap().as_str();
        let operation = if c.name("text").is_some() {
            FinderOperation::Text
        } else if c.name("html").is_some() {
            FinderOperation::Html
        } else if let Some(attr) = c.name("attr").map(|a| a.as_str()) {
            FinderOperation::Attr(attr)
        } else {
            // This should never happen, because we're guaranteed to have found a match for at
            // least one of the groups.
            panic!("Something went wrong, please provide {{text}}, {{html}}, or attr{{NAME}}");
        };

        let finder = Finder {
            operation,
            selector: Selector::parse(selector_str)
                .map_err(|e| format!("Bad CSS selector: {:?}", e.kind))?,
        };
        finders.push(finder);
    }

    if finders.is_empty() {
        Err("Please specify {text}, {html}, or attr{ATTRIBUTE}".to_string())
    } else {
        let document = Html::parse_document(&inputs.html);
        Ok(select_all(document, &finders))
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use super::*;

    #[test]
    fn test_showing_inner_text() {
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1 i {text}";
        let result = parse(build_inputs(html, selector));
        assert_eq!(result, Ok(vec!["world!".to_string()]));
    }

    #[test]
    fn test_bad_selector() {
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1^3 {text}";
        let err = parse(build_inputs(html, selector)).expect_err("not an Err");
        assert_eq!(true, err.starts_with("Bad CSS selector"));
    }

    #[test]
    fn test_showing_specific_attr() {
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1 attr{class}";
        let result = parse(build_inputs(html, selector));
        assert_eq!(result, Ok(vec!["foo".to_string()]));
    }

    #[test]
    fn test_multiple_selectors() {
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1 attr{class}, h1 {text}";
        let result = parse(build_inputs(html, selector));
        assert_eq!(
            result,
            Ok(vec!["foo".to_string(), "Hello, world!".to_string()])
        );
    }

    #[test]
    fn test_multiple_finders_on_same_node_shown_together() {
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h2 class="foo">Hello</h2>
            <h2 class="bar">Hi</h2>
        "#;
        let selector = "h2 attr{class}, h2 {text}";
        let result = parse(build_inputs(html, selector));
        // the class and text for a given node are shown together
        // i.e. it's class-text-class-text, rather than class-class-text-text
        let expected_result = vec![
            "foo".to_string(),
            "Hello".to_string(),
            "bar".to_string(),
            "Hi".to_string(),
        ];
        assert_eq!(result, Ok(expected_result));
    }

    #[test]
    fn test_html_operation() {
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello,<i>world!<strong>and more</strong></i><!--hello    --></h1>
        "#;
        let selector = "h1 {html}";
        let result = parse(build_inputs(html, selector));
        let expected_result = r#"
<h1 class="foo">
  Hello,
  <i>
    world!
    <strong>
      and more
    </strong>
  </i>
  <!-- hello -->
</h1>"#.trim_start().to_string();
        assert_eq!(result, Ok(vec![expected_result]));
    }

    #[test]
    fn test_html_operation_with_self_closing_tags() {
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello,<i>world!<strong>and more</strong></i><!--hello    --></h1>
        "#;
        let selector = "meta {html}";
        let result = parse(build_inputs(html, selector));
        let expected_result = r#"<meta charset="utf-8"></meta>"#.to_string();
        assert_eq!(result, Ok(vec![expected_result]));
    }

    #[test]
    fn test_html_operation_with_siblings() {
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <body>
                <div>foo</div>
                <div>bar</div>
            </body>
        "#;
        let selector = "body {html}";
        let result = parse(build_inputs(html, selector));
        let expected_result = r#"
<body>
  <div>
    foo
  </div>
  <div>
    bar
  </div>
</body>"#.trim_start().to_string();
        assert_eq!(result, Ok(vec![expected_result]));
    }

    #[test]
    fn test_bad_operation() {
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1";
        let result = parse(build_inputs(html, selector));
        assert_eq!(
            result,
            Err("Please specify {text}, {html}, or attr{ATTRIBUTE}".to_string())
        );
    }

    #[test]
    fn test_less_than_1024_bytes_of_html() {
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let result = read_from(Cursor::new(html));
        assert_eq!(result, Some(html.to_string()));
    }

    fn build_inputs(html: &str, selector: &str) -> Inputs {
        Inputs {
            html: html.to_string(),
            selector: selector.to_string(),
        }
    }
}
