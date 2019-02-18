
use std::process::exit;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use easycurses::{ColorPair, CursorVisibility, EasyCurses, Input, TimeoutMode};

use dictionary::{Entry, Text};



const FACES: [&str;6] = ["(ﾟДﾟ)" , "( ﾟД)" , "(  ﾟ)" , "(   )" , "(ﾟ  )" , "(Дﾟ )"];


pub fn main(rx: &Receiver<Option<Vec<Entry>>>, kuru: bool, bind_to: &str) {
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

        fn write(out: &mut EasyCurses, text: &str, color_pair: ColorPair, bold: bool) {
            out.set_color_pair(color_pair);
            if bold {
                out.set_bold(true);
            }
            out.print(text);
            if bold {
                out.set_bold(false);
            }
        }

        match text {
            Annot(s) => write(out, s, colorpair!(Yellow on Black), false),
            Countability(c) => write(out, &c.to_string(), colorpair!(Yellow on Black), false),
            Class(s) => write(out, s, colorpair!(Blue on Black), false),
            Definition(s) => write(out, s, colorpair!(White on Black), true),
            Example(s) => write(out, s, colorpair!(Green on Black), false),
            Information(s) => write(out, s, colorpair!(Cyan on Black), false),
            Note(s) => write(out, s, colorpair!(White on Black), false),
            Tag(s) => write(out, s, colorpair!(Red on Black), false),
            Word(s) => write(out, s, colorpair!(Black on Yellow), false),
        }
    }

    // DO NOT REMOVE THIS BLOCK (EasyCurses should finalize)
    {
        #![allow(clippy::unused_unit)]
        if_let_some!(mut out = EasyCurses::initialize_system(), ());

        out.set_cursor_visibility(CursorVisibility::Invisible);
        out.set_echo(false);
        out.set_input_timeout(TimeoutMode::Immediate);

        out.clear();
        out.set_color_pair(colorpair!(Black on White));
        out.print(concat!(env!("CARGO_PKG_NAME"), " v", env!("CARGO_PKG_VERSION")));
        out.set_color_pair(colorpair!(White on Black));
        out.print(format!("\non {}", bind_to));
        out.print("\n\npress q to quit");
        out.refresh();

        let timeout = Duration::from_millis(100);
        let mut face_index = 0;
        let mut face_col = 0;
        let mut face_back = false;
        let mut rc = (0, 0);
        let mut bullets = vec![];

        loop {
            while let Ok(entries) = rx.recv_timeout(timeout) {
                out.clear();
                if let Some(entries) = entries {
                    for entry in entries {
                        color_key(&mut out, &entry.key);
                        for definition in &entry.definitions {
                            for (index, text) in definition.content.iter().enumerate() {
                                if 0 < index {
                                    out.print(" ");
                                }
                                color(&mut out, text);
                            }
                            out.print("\n");
                        }
                    }
                } else {
                    out.set_color_pair(colorpair!(White on Red));
                    out.set_bold(true);
                    out.print("Not Found");
                    out.set_bold(false);
                }
                rc = out.get_cursor_rc();
                out.refresh();
            }

            if let Some(input) = out.get_input() {
                match input {
                    Input::Character(' ') if kuru => bullets.push((face_col + 2, 0)),
                    Input::Character('q') => break,
                    _ => (),
                }
            }

            // Kuru-Kuru Face
            if kuru {
                let (row, _col) = rc;
                let (rows, cols) = out.get_row_col_count();

                if !bullets.is_empty() {
                    for (bc, br) in &mut bullets {
                        out.move_rc(rows - *br - 1, *bc);
                        out.delete_char();
                        out.insert_char(' ');
                        if *br < rows {
                            *br += 1;
                        }
                        if *br < rows {
                            out.move_rc(rows - *br - 1, *bc);
                            out.delete_char();
                            out.insert_char('o');
                        }
                    }
                    bullets.retain(|(_, r)| *r < rows);
                }

                if rows <= row + 1 {
                    continue;
                }

                out.set_color_pair(colorpair!(White on Black));
                out.move_rc(rows - 1, face_col);
                out.delete_line();
                out.print(FACES[face_index]);
                out.refresh();

                face_col += if face_back { -1 } else { 1 };
                if cols - 6 < face_col || face_col == 0 {
                    face_back = !face_back;
                }
                {
                    #![allow(clippy::collapsible_if)]
                    face_index = if face_back {
                        if face_index == 0 { FACES.len() - 1 } else { face_index - 1 }
                    } else {
                        if FACES.len() - 1 <= face_index { 0 } else { face_index + 1 }
                    };
                }
            }
        }

        out.clear();
        out.refresh();
    }

    exit(0);
}
