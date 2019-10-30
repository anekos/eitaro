use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, stdout, Stdout};
use std::path::Path;

use crate::errors::AppResultU;
use crate::dictionary::Dictionary;
use super::Exporter;



const MAX_LENGTH: usize = 500;


pub fn export<T: AsRef<Path>>(dictionary_path: &T) -> AppResultU {
    let mut dic = Dictionary::new(dictionary_path);

    let out = stdout();
    let out = BufWriter::new(out);
    let mut out = csv::Writer::from_writer(out);

    for level in 0..255 {
        read_level(&mut dic, dictionary_path, level, &mut out)?;
    }

    out.flush()?;

    Ok(())
}


fn read_level<T: AsRef<Path>>(dic: &mut Dictionary, dictionary_path: &T, level: u8, out: &mut csv::Writer<BufWriter<Stdout>>) -> AppResultU {
    let mut path = dictionary_path.as_ref().to_path_buf();
    path.push(format!("level-{}", level));

    if !path.is_file() {
        return Ok(());
    }

    let file = File::open(&path)?;
    let file = BufReader::new(file);
    let svl = format!("SVL{}", level);

    for word in file.lines() {
        let word = word?;
        if let Some(entries) = dic.get(&word)? {
            let mut buf = "".to_owned();

            'outer: for entry in entries {
                for def in entry.definitions {
                    for t in def.content {
                        use crate::dictionary::Text::*;

                        if let Definition(s) = &t {
                            if !buf.is_empty() {
                                buf.push(' ');
                            }
                            buf.push_str(s);
                            if MAX_LENGTH < buf.len() {
                                break 'outer;
                            }
                        }
                    }
                }
            }

            out.write_record(&[&word, &buf, &svl])?;
        }
    }

    Ok(())
}
