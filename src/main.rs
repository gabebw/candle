use scraper::{Html, Selector};
use std::env;
use std::io::{self, Read};

fn read_from_stdin() -> Option<String> {
    // It might not be valid UTF-8, so read to a vector of bytes and convert it to UTF-8, lossily
    let mut buffer: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut buffer).ok()?;
    Some(String::from_utf8_lossy(&buffer).to_string())
}

fn main() {
    let selector = env::args().nth(1).unwrap();

    if let Some(html) = read_from_stdin() {
        match parse(&html, &selector) {
            Ok(result) => {
                for r in result {
                    println!("{}", &r.trim());
                }
            },
            Err(e) => eprintln!("{}", e)
        }
    } else {
        eprintln!("Couldn't read from STDIN");
    }
}

fn parse(html: &str, selector: &str) -> Result<Vec<String>, String> {
    let document = Html::parse_document(html);
    let selector = Selector::parse(selector)
        .map_err(|e| format!("Bad CSS selector: {:?}", e.kind))?;

    Ok(document.select(&selector).map(|element| element.text().collect()).collect())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parsing_html_with_selector(){
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1 i";
        let result = parse(html, selector);
        assert_eq!(result, Ok(vec!("world!".to_string())));
    }

    #[test]
    fn test_parsing_html_with_bad_selector(){
        let html = r#"
            <!DOCTYPE html>
            <meta charset="utf-8">
            <title>Hello, world!</title>
            <h1 class="foo">Hello, <i>world!</i></h1>
        "#;
        let selector = "h1^3";
        let err = parse(html, selector).expect_err("not an Err");
        assert_eq!(true, err.starts_with("Bad CSS selector"));
    }
}
