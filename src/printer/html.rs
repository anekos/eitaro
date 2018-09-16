
use std::fmt::Write;

use errors::AppError;
use printer::parser::{Text, parse};
use htmlescape::encode_minimal;



pub fn generate(s: &str) -> Result<String, AppError> {
    let mut result = "<html><head><title>Eitaro</title></head><body>\n".to_owned();

    for line in s.lines() {
        let text = parse(line)?;
        for it in &text {
            write_html(&mut result, it);
        }
    }

    result.push_str("</body></html>");

    Ok(result)
}

fn write_html(out: &mut String, text: &Text) {
    fn span(out: &mut String, class: &str, text: &str) {
        write!(out, "<span class=\"{}\">{}</span>", encode_minimal(class), encode_minimal(text)).unwrap();
    }

    use self::Text::*;

    match text {
        Annot(s) => span(out, "annotation", s),
        Class(s) => span(out, "class", s),
        Example(s) => span(out, "example", s),
        Note(s) => span(out, "note", s),
        Plain(s) => span(out, "plain", s),
        Tag(s) => span(out, "tag", s),
        Word(s) => write!(out, "<h1>{}</h1>\n", encode_minimal(s)).unwrap(),
    }
}
