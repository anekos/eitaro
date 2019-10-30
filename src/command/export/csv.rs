use std::io::Write;

use crate::errors::AppResultU;
use crate::dictionary::Dictionary;
use super::Exporter;



const MAX_LENGTH: usize = 500;

pub struct CsvExporter();

impl Exporter for CsvExporter {
    fn export<T: Write>(&self, dictionary: &mut Dictionary, words: &[&str], out: &mut T) -> AppResultU {
        let mut out = csv::Writer::from_writer(out);

        for word in words {
            if let Some(entries) = dictionary.get(&word)? {
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

                out.write_record(&[word, buf.as_str()])?;
            } else {
                eprintln!("Definition not found: {}", word);
            }
        }

        out.flush()?;
        Ok(())
    }
}
