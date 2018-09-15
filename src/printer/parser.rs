
use pom::parser::*;
use pom::{Parser, TextInput};



#[derive(PartialEq, Eq, Debug)]
pub enum Text {
    Annot(String),
    Class(String),
    Example(String),
    Note(String),
    Plain(String),
    Tag(String),
    Word(String),
}


const SPECIALS: &str = "{}〈〉《》◆■";


pub fn parse(input: &str) -> Result<Vec<Text>, String> {
    let mut input = TextInput::new(input);
    text().parse(&mut input).map_err(|it| it.to_string())
}

fn text() -> Parser<char, Vec<Text>> {
    let p = annot() | class() | example() | tag() | word() | note() | plain();
    p.repeat(0..)
}

fn annot() -> Parser<char, Text> {
    let p = sym('〈') * none_of("〈〉").repeat(1..) - sym('〉');
    p.map(|it| Text::Annot(v2s(it)))
}

fn class() -> Parser<char, Text> {
    let p = sym('《') * none_of("《》").repeat(1..) - sym('》');
    p.map(|it| Text::Class(v2s(it)))
}

fn example() -> Parser<char, Text> {
    let p = seq("■・") * none_of(SPECIALS).repeat(1..);
    p.map(|it| Text::Example(v2s(it)))
}

fn note() -> Parser<char, Text> {
    let p = sym('◆') * none_of(SPECIALS).repeat(1..);
    p.map(|it| Text::Note(v2s(it)))
}

fn plain() -> Parser<char, Text> {
    let p = none_of(SPECIALS).repeat(1..);
    p.map(|it| Text::Plain(v2s(it)))
}

fn tag() -> Parser<char, Text> {
    let p = sym('{') * none_of("{}").repeat(1..) - sym('}');
    p.map(|it| Text::Tag(v2s(it)))
}

fn v2s(s: Vec<char>) -> String {
    s.into_iter().collect()
}

fn word() -> Parser<char, Text> {
    let p = seq("#") * none_of(SPECIALS).repeat(1..);
    p.map(|it| Text::Word(v2s(it)))
}



#[cfg(test)]#[test]
fn test_parser() {
    assert_eq!(
        parse("{foo}"),
        Ok(vec![Text::Tag("foo".to_string())]));

    assert_eq!(
        parse("{foo} plain hoge"),
        Ok(vec![
           Text::Tag("foo".to_string()),
           Text::Plain(" plain hoge".to_string())]));
}
