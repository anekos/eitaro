
use pom::parser::*;
use pom::{Parser, TextInput};



#[derive(PartialEq, Eq, Debug)]
pub enum Text {
    Annot(String),
    Class(String),
    Definition(String),
    Example(String),
    Information(String),
    LineBreak,
    Note(String),
    Tag(String),
    Word(String),
}


const SPECIALS: &str = "{}〈〉《》◆■〔〕\n";


pub fn parse(input: &str) -> Result<Vec<Text>, String> {
    let mut input = TextInput::new(input);
    text().parse(&mut input).map_err(|it| it.to_string())
}

fn text() -> Parser<char, Vec<Text>> {
    let p = annot() | class() | example() | tag() | word() | information() | note() | definition() | line_break();
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
    let p1 = sym('■');
    let p2 = sym('・') * none_of(SPECIALS).repeat(1..);
    let p2 = p2.map(|it| Text::Example(v2s(it)));
    let p3 = none_of(SPECIALS).repeat(1..);
    let p3 = p3.map(|it| Text::Definition(format!("■{}", v2s(it))));
    p1 * (p2 | p3)
}

fn line_break() -> Parser<char, Text> {
    let p = seq("\n");
    p.map(|_| Text::LineBreak)
}

fn note() -> Parser<char, Text> {
    let p = sym('〔') * none_of("〔〕").repeat(1..) - sym('〕');
    p.map(|it| Text::Note(v2s(it)))
}

fn information() -> Parser<char, Text> {
    let p = sym('◆') * none_of(SPECIALS).repeat(1..);
    p.map(|it| Text::Information(v2s(it)))
}

fn definition() -> Parser<char, Text> {
    let p = none_of(SPECIALS).repeat(1..);
    p.map(|it| Text::Definition(v2s(it)))
}

fn tag() -> Parser<char, Text> {
    let p = sym('{') * none_of("{}").repeat(1..) - sym('}');
    p.map(|it| Text::Tag(v2s(it)))
}

fn v2s(s: Vec<char>) -> String {
    s.into_iter().collect()
}

fn word() -> Parser<char, Text> {
    let p = seq("#") * sym(' ').repeat(0..) * none_of("\n").repeat(1..) - seq("\n");
    p.map(|it| Text::Word(v2s(it)))
}



#[cfg(test)]#[test]
fn test_parser() {
    assert_eq!(
        parse("{foo}"),
        Ok(vec![Text::Tag("foo".to_string())]));

    assert_eq!(
        parse("{foo} definition hoge"),
        Ok(vec![
           Text::Tag("foo".to_string()),
           Text::Definition(" definition hoge".to_string())]));

    assert_eq!(
        parse("■meow :"),
        Ok(vec![
           Text::Definition("■meow :".to_string())]));
}
