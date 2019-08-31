use regex::Regex;
use scraper::{Html, Selector};
use std::env;
use std::io::{self, Read};
use std::process;

struct Inputs {
    selector: String,
    html: String
}

fn read_from_stdin() -> Option<String> {
    // It might not be valid UTF-8, so read to a vector of bytes and convert it to UTF-8, lossily
    let mut buffer: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut buffer).ok()?;
    Some(String::from_utf8_lossy(&buffer).to_string())
}

fn read_inputs() -> Result<Inputs, String> {
    let selector = env::args().nth(1).ok_or("Usage: candle SELECTOR")?;
    let html = read_from_stdin().ok_or("Error: couldn't read from STDIN")?;
    Ok(Inputs { selector, html })
}

fn main() {
    match read_inputs() {
        Ok(inputs) => {
            match parse(&inputs.html, &inputs.selector) {
                Ok(result) => {
                    for r in result {
                        println!("{}", &r.trim());
                    }
                },
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    }
}

fn parse(html: &str, selector: &str) -> Result<Vec<String>, String> {
    let document = Html::parse_document(html);
    let re = Regex::new(r"(?P<selector>.+) (?:(?P<text>\{text\})|(attr\{(?P<attr>[^}]+)\}))$").unwrap();
    let captures = re.captures(selector).unwrap();
    let selector = Selector::parse(captures.name("selector").unwrap().as_str())
        .map_err(|e| format!("Bad CSS selector: {:?}", e.kind))?;
    let selected = document.select(&selector);

    if let Some(_) = captures.name("text") {
        Ok(selected.map(|element| element.text().collect()).collect())
    } else if let Some(attr) = captures.name("attr") {
        Ok(selected
            .filter_map(|element| element.value().attr(attr.as_str()).map(|s| s.to_string()))
            .collect())
    } else {
        Err("Please specify {text} or attr{ATTRIBUTE}".to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_showing_inner_text(){
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1 i {text}";
        let result = parse(html, selector);
        assert_eq!(result, Ok(vec!("world!".to_string())));
    }

    #[test]
    fn test_bad_selector(){
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1^3 {text}";
        let err = parse(html, selector).expect_err("not an Err");
        assert_eq!(true, err.starts_with("Bad CSS selector"));
    }

    #[test]
    fn test_showing_specific_attr(){
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1 attr{class}";
        let result = parse(html, selector);
        assert_eq!(result, Ok(vec!("foo".to_string())));
    }
}
