
use std::fmt::{Error as FmtError, Write};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread::spawn;

use easycurses::{ColorPair, EasyCurses};

mod parser;

use dictionary::Entry;
use self::parser::{parse, Text};
use errors::{AppError};



pub struct Screen {
    tx: SyncSender<Option<Vec<Entry>>>,
}

impl Screen {
    pub fn new(curses: bool) -> Self {
        let (tx, rx) = sync_channel(0);

        if curses {
            spawn(|| curses_main(rx));
        } else {
            spawn(|| standard_main(rx));
        }

        Screen { tx }
    }

    pub fn print_opt(&self, content: Option<Vec<Entry>>) {
        self.tx.send(content).unwrap();
    }
}


pub fn print_opt(entries: Option<Vec<Entry>>) -> Result<(), AppError> {
    use colored::*;

    fn color_key(out: &mut String, key: &str) -> Result<(), FmtError> {
        write!(out, "{}\n", key.black().on_yellow().bold())
    }

    fn color(out: &mut String, text: &Text) -> Result<(), FmtError> {
        use self::Text::*;

        match text {
            Annot(s) => write!(out, "{} ", s.yellow()),
            Class(s) => write!(out, "{} ", s.blue()),
            Example(s) => write!(out, " {} ", s.green()),
            LineBreak => writeln!(out),
            Note(s) => write!(out, " {}", s.cyan()),
            Plain(s) => write!(out, "{}", s.white().bold()),
            Tag(s) => write!(out, "{}", s.red().bold()),
            Word(s) => color_key(out, &s),
        }
    }


    if let Some(entries) = entries {
        for entry in entries {
            let mut buffer = "".to_owned();
            let texts = parse(&entry.content)?;
            color_key(&mut buffer, &entry.key)?;
            for text in &texts {
                color(&mut buffer, text)?;
            }
            print!("{}", buffer);
        }
    } else {
        println!("{}", "Not Found".black().on_red());
    }

    Ok(())
}


fn curses_main(rx: Receiver<Option<Vec<Entry>>>) {
    use easycurses::Color::*;

    fn color_key(out: &mut EasyCurses, key: &str) {
        out.set_color_pair(colorpair!(Black on Yellow));
        out.set_bold(true);
        out.print(key);
        out.print("\n");
        out.set_bold(false);
    }

    fn color(out: &mut EasyCurses, text: &Text) {
        use self::Text::*;

        fn write(out: &mut EasyCurses, prefix: &str, text: &str, suffix: &str, color_pair: ColorPair, bold: bool) {
            out.print(prefix);
            out.set_color_pair(color_pair);
            if bold {
                out.set_bold(true);
            }
            out.print(text);
            if bold {
                out.set_bold(false);
            }
            out.print(suffix);
        }

        match text {
            Annot(s) => write(out, "", s, " ", colorpair!(Yellow on Black), false),
            Class(s) => write(out, "", s, " ", colorpair!(Blue on Black), false),
            Example(s) => write(out, " ", s, " ", colorpair!(Green on Black), false),
            LineBreak => write(out, "", "\n", "", colorpair!(Black on Black), false),
            Note(s) => write(out, " ", s, "", colorpair!(Cyan on Black), false),
            Plain(s) => write(out, "", s, "", colorpair!(White on Black), true),
            Tag(s) => write(out, "", s, "", colorpair!(Red on Black), false),
            Word(s) => write(out, "", s, "", colorpair!(Black on Yellow), false),
        }
    }

    if_let_some!(mut out = EasyCurses::initialize_system(), ());

    for entries in rx {
        out.clear();
        if let Some(entries) = entries {
            for entry in entries {
                let texts = parse(&entry.content).unwrap(); // FIXME
                color_key(&mut out, &entry.key);
                for text in &texts {
                    color(&mut out, text);
                }
            }
        } else {
            out.set_color_pair(colorpair!(White on Red));
            out.set_bold(true);
            out.print("Not Found");
            out.set_bold(false);
        }
        out.refresh();
    }
}

fn standard_main(rx: Receiver<Option<Vec<Entry>>>) {
    for entries in rx {
        print_opt(entries).unwrap();
    }
}
