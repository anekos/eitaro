
use colored::*;

pub mod parser;

use self::parser::{Text, parse};


pub fn print_colored(s: &str) {
    let text = parse(s).unwrap(); // FIXME

    for it in &text {
        print_text(it);
    }
}

fn print_text(text: &Text) {
    use self::Text::*;

    match text {
        Annot(s) => print!("{} ", s.yellow()),
        Class(s) => print!("{} ", s.blue()),
        Example(s) => print!(" {} ", s.green()),
        Note(s) => print!(" {}", s.cyan()),
        Plain(s) => print!("{}", s.white().bold()),
        Tag(s) => print!("{}", s.red().bold()),
    }
}
