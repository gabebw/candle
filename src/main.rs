use encoding_rs::Encoding;
use isatty::stdin_isatty;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
use std::cmp;
use std::env;
use std::io::{self, ErrorKind, Read, Write};
use std::process;

mod tree;

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

impl<'a> Finder<'a> {
    fn match_and_apply(&self, element: &ElementRef) -> Option<String> {
        if self.selector.matches(element) {
            match self.operation {
                FinderOperation::Text => Some(element.text().collect()),
                FinderOperation::Attr(attr) => element.value().attr(attr).map(|s| s.to_string()),
                FinderOperation::Html => Some(tree::print_tree(*element, 0))
            }
        } else {
            None
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
    let selector = env::args().nth(1).unwrap_or_else(|| "".to_string());
    let html = read_from(io::stdin()).ok_or("Error: couldn't read from STDIN")?;
    Ok(Inputs { selector, html })
}

fn cleanly_write(content: &str) {
    let mut stdout = io::stdout();

    if let Err(e) = writeln!(stdout, "{}", content) {
        // Ignore broken pipes, they most likely come from piping to something
        // that truncates the output, like `head`.
        if e.kind() != ErrorKind::BrokenPipe {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn main() {
    if stdin_isatty() {
        eprintln!("You must pipe in input to candle");
        process::exit(1);
    }

    match read_inputs() {
        Ok(inputs) => match parse(inputs) {
            Ok(result) => {
                for r in result {
                    cleanly_write(&r.trim());
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
    for element in html.tree.nodes().filter_map(ElementRef::wrap) {
        for value in finders.iter().filter_map(|finder| finder.match_and_apply(&element)) {
            results.push(value);
        }
    }
    results
}

fn finders<'a>(inputs: &'a Inputs) -> Result<Vec<Finder<'a>>, String> {
    if inputs.selector.is_empty() {
        let finder = Finder {
            selector: Selector::parse("html").unwrap(),
            operation: FinderOperation::Html
        };
        Ok(vec![finder])
    } else {
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
                    .map_err(|e| format!("'{}' is a bad CSS selector: {:?}", selector_str.trim(), e.kind))?,
            };
            finders.push(finder);
        }
        Ok(finders)
    }
}

fn parse(inputs: Inputs) -> Result<Vec<String>, String> {
    let finders = finders(&inputs)?;
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
        assert!(err.starts_with("'h1^3' is a bad CSS selector:"));
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
    fn test_trim_start_n_exact_spaces(){
        let four_spaces = format!("{:n$}", "", n = 4);
        let input = format!("{}there", four_spaces);
        let result = tree::trim_start_n(&input, 4);

        assert_eq!(result, "there");
    }

    #[test]
    fn test_trim_start_n_extra_space(){
        let five_spaces = format!("{:n$}", "", n = 5);
        let input = format!("{}there", five_spaces);
        let result = tree::trim_start_n(&input, 4);

        assert_eq!(result, " there");
    }

    #[test]
    fn test_trim_start_n_trimming_more_than_the_length_of_the_string(){
        let one_space = " ";
        let input = format!("{}there", one_space);
        let result = tree::trim_start_n(&input, 10);

        assert_eq!(result, "there");
    }

    #[test]
    fn test_printing_indented_js(){
        let input = r#"
            <body>
            <div>
                <div>
                <script>
                    var foo = "foo";
                        var bar = "bar";
                function x(){
                      return true;
                    }
                </script>
                </div>
            </div>
            </body>
        "#;
        let output = r#"
<body>
  <div>
    <div>
      <script>
        var foo = "foo";
            var bar = "bar";
        function x(){
          return true;
        }
      </script>
    </div>
  </div>
</body>"#.trim_start().to_string();
        let selector = "body {html}";
        let result = parse(build_inputs(input, selector));
        assert_eq!(result, Ok(vec![output]));
    }

    #[test]
    fn test_empty_script_tag(){
        let input = r#"<body><script></script></body>"#;
        let output = r#"
<body>
  <script>
  </script>
</body>"#.trim_start().to_string();
        let selector = "body {html}";
        let result = parse(build_inputs(input, selector));
        assert_eq!(result, Ok(vec![output]));
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
