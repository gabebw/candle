use encoding_rs::Encoding;
use isatty::stdin_isatty;
use regex::Regex;
use scraper::{Html, Selector};
use std::env;
use std::io::{self, Read};
use std::process;

struct Inputs {
    selector: String,
    html: String,
    raw_bytes: Vec<u8>
}

fn read_from_stdin() -> Option<(String, Vec<u8>)> {
    // It might not be valid UTF-8, so read to a vector of bytes and convert it to UTF-8, lossily
    let mut buffer: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut buffer).ok()?;
    let string = String::from_utf8_lossy(&buffer).to_string();
    Some((string, buffer))
}

fn read_inputs() -> Result<Inputs, String> {
    let selector = env::args().nth(1).ok_or("Usage: candle SELECTOR")?;
    let (html, raw_bytes) = read_from_stdin().ok_or("Error: couldn't read from STDIN")?;
    Ok(Inputs { selector, html, raw_bytes })
}

fn main() {
    if stdin_isatty() {
        eprintln!("You must pipe in input to candle");
        process::exit(1);
    }

    match read_inputs() {
        Ok(inputs) => {
            match parse(inputs) {
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

fn select(document: scraper::Html, captures: regex::Captures) -> Result<Vec<String>, String> {
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
        Err("Unknown request".to_string())
    }
}

fn detect_encoding_and_re_parse(document: scraper::Html, inputs: &Inputs) -> scraper::Html {
    let meta_selector = Selector::parse("meta[charset]").unwrap();
    // If there's a `<meta charset="...">` tag, re-parse the HTML in that encoding.
    // Otherwise, keep it exactly the same.
    if let Some(meta_result) = document.select(&meta_selector).nth(0) {
        let charset = meta_result.value().attr("charset").unwrap();
        match Encoding::for_label(charset.as_bytes()) {
            Some(encoding) => Html::parse_document(&*encoding.decode(&inputs.raw_bytes).0),
            None => document
        }
    } else {
        document
    }
}

fn parse(inputs: Inputs) -> Result<Vec<String>, String> {
    let document = detect_encoding_and_re_parse(Html::parse_document(&inputs.html), &inputs);
    let re = Regex::new(r"(?P<selector>.+) (?:(?P<text>\{text\})|(attr\{(?P<attr>[^}]+)\}))$").unwrap();
    match re.captures(&inputs.selector) {
        Some(captures) => select(document, captures),
        None => {
            Err("Please specify {text} or attr{ATTRIBUTE}".to_string())
        }
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

    #[test]
    fn test_no_text_or_attr_specification(){
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1";
        let result = parse(html, selector);
        assert_eq!(result, Err("Please specify {text} or attr{ATTRIBUTE}".to_string()));
    }
}
