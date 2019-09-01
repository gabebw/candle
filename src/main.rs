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
}

struct Finder<'a> {
    selector: &'a str,
    attr: Option<&'a str>,
    is_text: bool,
}

fn read_from_stdin() -> Option<String> {
    // It might not be valid UTF-8, so read to a vector of bytes and convert it to UTF-8, lossily
    let mut buffer: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut buffer).ok()?;
    let re_meta_charset = Regex::new(r#"<meta\s+charset=["']([^'"]+)["']"#).unwrap();
    let string = String::from_utf8_lossy(&buffer).to_string();
    if let Some(captures) = re_meta_charset.captures(&string[..1024]) {
        let charset = captures.get(1).unwrap().as_str();
        match Encoding::for_label(charset.as_bytes()) {
            Some(encoding) => {
                let string_with_new_encoding = encoding.decode(&buffer).0;
                Some((*string_with_new_encoding).to_string())
            }
            None => Some(string)
        }
    } else {
        Some(string)
    }
}

fn read_inputs() -> Result<Inputs, String> {
    let selector = env::args().nth(1).ok_or("Usage: candle SELECTOR")?;
    let html = read_from_stdin().ok_or("Error: couldn't read from STDIN")?;
    Ok(Inputs { selector, html })
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

fn select_all(document: scraper::Html, finders: Vec<Finder>) -> Result<Vec<String>, String> {
    let mut results: Vec<String> = Vec::new();
    for finder in finders {
        results.extend(select(&document, &finder)?);
    }
    Ok(results)
}

fn select(document: &scraper::Html, finder: &Finder) -> Result<Vec<String>, String> {
    let selector = Selector::parse(finder.selector)
        .map_err(|e| format!("Bad CSS selector: {:?}", e.kind))?;
    let selected = document.select(&selector);

    if finder.is_text {
        Ok(selected.map(|element| element.text().collect()).collect())
    } else if let Some(attr) = finder.attr {
        Ok(selected
            .filter_map(|element| element.value().attr(attr).map(|s| s.to_string()))
            .collect())
    } else {
        Err("Unknown finder (not `{text}` or `attr{...}`".to_string())
    }
}

fn parse(inputs: Inputs) -> Result<Vec<String>, String> {
    let document = Html::parse_document(&inputs.html);
    let re = Regex::new(r"(?P<selector>[^{}]+) (?:(?P<text>\{text\})|(attr\{(?P<attr>[^}]+)\}))[,]?\s*").unwrap();
    let finders: Vec<Finder> = re.captures_iter(&inputs.selector)
        .map(|c| Finder { selector: c.name("selector").unwrap().as_str(), is_text: c.name("text").is_some(), attr: c.name("attr").map(|a| a.as_str()) }).collect();

    if finders.len() == 0 {
        Err("Please specify {text} or attr{ATTRIBUTE}".to_string())
    } else {
        select_all(document, finders)
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
        let result = parse(build_inputs(html, selector));
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
        let err = parse(build_inputs(html, selector)).expect_err("not an Err");
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
        let result = parse(build_inputs(html, selector));
        assert_eq!(result, Ok(vec!("foo".to_string())));
    }

    #[test]
    fn test_multiple_selectors(){
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1 attr{class}, h1 {text}";
        let result = parse(build_inputs(html, selector));
        assert_eq!(result, Ok(vec!("foo".to_string(), "Hello, world!".to_string())));
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
        let result = parse(build_inputs(html, selector));
        assert_eq!(result, Err("Please specify {text} or attr{ATTRIBUTE}".to_string()));
    }

    fn build_inputs(html: &str, selector: &str) -> Inputs {
        Inputs {
            html: html.to_string(),
            selector: selector.to_string(),
        }
    }
}
