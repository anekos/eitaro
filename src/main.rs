
use std::process::exit;

mod args;
mod command;
mod dictionary;
mod errors;
mod loader;
mod parser;
mod path;
mod screen;
mod str_utils;

use crate::errors::{AppError, AppResultU};
use crate::command::http::{Config as HttpConfig};



const DEFAULT_PROMPT: &str = "Eitaro> ";


fn _main() -> AppResultU {
    let app = args::build();

    let matches = app.get_matches();

    let dictionary_path = path::get_dictionary_path()?;

    if let Some(ref matches) = matches.subcommand_matches("build") {
        let files: Vec<&str> = matches.values_of("dictionary-file").unwrap().collect(); // Required
        command::builder::build_dictionary(&files, &dictionary_path)
    } else if let Some(ref matches) = matches.subcommand_matches("export") {
        let as_text = matches.is_present("as-text");
        command::export::export(&dictionary_path, as_text)
    } else if let Some(ref matches) = matches.subcommand_matches("html") {
        let word = matches.value_of("word").unwrap(); // Required
        command::html::lookup(&dictionary_path, word)
    } else if let Some(ref matches) = matches.subcommand_matches("interactive") {
        command::lookup::shell(&dictionary_path, matches.value_of("prompt").unwrap_or(DEFAULT_PROMPT))
    } else if let Some(ref matches) = matches.subcommand_matches("lemmatize") {
        let word = matches.value_of("word").unwrap(); // Required
        command::lemmatize::lemmatize(&dictionary_path, word)
    } else if let Some(ref matches) = matches.subcommand_matches("lookup") {
        let word = matches.value_of("word").unwrap(); // Required
        let n = matches.value_of("n").map(|it| it.parse()).transpose()?;
        let color = !matches.is_present("no-color");
        command::lookup::lookup(&dictionary_path, word, color, n)
    } else if let Some(ref matches) = matches.subcommand_matches("server") {
        let bind_to = matches.value_of("bind-to").unwrap_or("127.0.0.1:8116");
        command::http::start_server(
            bind_to,
            HttpConfig {
                curses: matches.is_present("curses"),
                dictionary_path,
                do_print: matches.is_present("print"),
                ignore_not_found: matches.is_present("ignore"),
                kuru: matches.is_present("kuru"),
            })?;
        Ok(())
    } else {
        command::lookup::shell(&dictionary_path, DEFAULT_PROMPT)
    }
}

fn main() {
    use failure::Fail;

    match _main() {
        Err(AppError::Void) | Ok(_) => (),
        Err(err) => {
            let mut fail: &dyn Fail = &err;
            let mut message = err.to_string();

            while let Some(cause) = fail.cause() {
                message.push_str(&format!("\n\tcaused by: {}", cause));
                fail = cause;
            }

            eprintln!("Error: {}", message);

            exit(1);

        }
    }
}
