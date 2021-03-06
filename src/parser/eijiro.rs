
use pom::parser::*;
use pom::{Parser, TextInput};

use crate::dictionary::Text;
use crate::parser::utils::*;



const SPECIALS: &str = "{}〈〉《》◆■〔〕\n";


pub fn parse_line(input: &str) -> Result<Vec<Text>, pom::Error> {
    let mut input = TextInput::new(input);
    text().parse(&mut input)
}

fn with_spaces(p: Parser<char, Text>) -> Parser<char, Text> {
    sym(' ').repeat(0..) * p - sym(' ').repeat(0..)
}

fn text() -> Parser<char, Vec<Text>> {
    let p = annot() | class() | example() | etymology() | tag() | word() | information() | note() | definition();
    let p = with_spaces(p);
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

fn etymology() -> Parser<char, Text> {
    let p = seq("【語源】") * none_of(SPECIALS).repeat(1..);
    p.map(|it| Text::Etymology(v2s(it)))
}

fn tag() -> Parser<char, Text> {
    let p = sym('{') * none_of("{}").repeat(1..) - sym('}');
    p.map(|it| Text::Tag(v2s(it)))
}

fn word() -> Parser<char, Text> {
    let p = seq("#") * sym(' ').repeat(0..) * none_of("\n").repeat(1..) - seq("\n");
    p.map(|it| Text::Word(v2s(it)))
}



#[cfg(test)]#[test]
fn test_parser() {
    assert_eq!(
        parse_line("{foo}"),
        Ok(vec![Text::Tag("foo".to_string())]));

    assert_eq!(
        parse_line("{foo} definition hoge"),
        Ok(vec![
           Text::Tag("foo".to_string()),
           Text::Definition("definition hoge".to_string())]));

    assert_eq!(
        parse_line(" 〈米俗〉ブラブラする"),
        Ok(vec![
           Text::Annot("米俗".to_string()),
           Text::Definition("ブラブラする".to_string())]));

    assert_eq!(
        parse_line("{自動} 〈米俗〉ブラブラする"),
        Ok(vec![
           Text::Tag("自動".to_string()),
           Text::Annot("米俗".to_string()),
           Text::Definition("ブラブラする".to_string())]));

    assert_eq!(
        parse_line("■meow :"),
        Ok(vec![
           Text::Definition("■meow :".to_string())]));
}
