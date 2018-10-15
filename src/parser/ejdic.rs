
use pom::parser::*;
use pom::{Parser, TextInput};

use dictionary::Text;
use parser::utils::*;



const SPECIALS: &str = "{}〈〉《》◆■〔〕\n";


pub fn parse_line(input: &str) -> Result<Vec<Text>, String> {
    let mut input = TextInput::new(input);
    text().parse(&mut input).map_err(|it| it.to_string())
}

fn with_spaces(p: Parser<char, Text>) -> Parser<char, Text> {
    sym(' ').repeat(0..) * p - sym(' ').repeat(0..)
}

fn text() -> Parser<char, Vec<Text>> {
    // let p = annot() | class() | example() | tag() | word() | information() | note() | definition();
    let p = note() | countability() | definition();
    let p = with_spaces(p);
    p.repeat(0..)
}

fn countability() -> Parser<char, Text> {
    let q1 = (sym('U') | sym('C')).map(Text::Countability);
    let q2 = none_of("〈〉").repeat(1..).map(|it| Text::Note(v2s(it)));
    sym('〈') * (q1 | q2) - sym('〉')
}

fn note() -> Parser<char, Text> {
    let p = sym('〔') * none_of("〔〕").repeat(1..) - sym('〕');
    p.map(|it| Text::Note(v2s(it)))
}

fn definition() -> Parser<char, Text> {
    let p = none_of(SPECIALS).repeat(1..);
    p.map(|it| Text::Definition(v2s(it)))
}
