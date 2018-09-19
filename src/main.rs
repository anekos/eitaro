
#[macro_use] extern crate clap;
#[macro_use] extern crate failure_derive;
#[macro_use] extern crate if_let_return;
extern crate app_dirs;
extern crate colored;
extern crate encoding;
extern crate failure;
extern crate hyper;
extern crate kv;
extern crate nickel;
extern crate percent_encoding;
extern crate pom;
extern crate readline;
extern crate unicase;

use std::process::exit;

use clap::{Arg, SubCommand};

mod command;
mod dictionary;
mod errors;
mod loader;
mod path;
mod printer;
mod str_utils;

use errors::AppError;
use command::http::{Config as HttpConfig};



fn _main() -> Result<(), AppError> {
    let app = app_from_crate!()
        .subcommand(SubCommand::with_name("build")
                    .alias("b")
                    .about("Build dictionary")
                    .arg(Arg::with_name("dictionary-path")
                         .help("Dictionary file path")
                         .required(true)))
        .subcommand(SubCommand::with_name("color")
                    .alias("c")
                    .about("Color dictionary text from STDIN"))
        .subcommand(SubCommand::with_name("lookup")
                    .alias("l")
                    .about("Lookup")
                    .arg(Arg::with_name("word")
                         .help("Word")
                         .required(true)))
        .subcommand(SubCommand::with_name("server")
                    .alias("s")
                    .about("HTTP Server")
                    .arg(Arg::with_name("clear")
                         .help("Clear before print")
                         .short("c")
                         .long("clear"))
                    .arg(Arg::with_name("ignore")
                         .help("Ignore not found")
                         .short("i")
                         .long("ignore"))
                    .arg(Arg::with_name("print")
                         .help("Print result stdout")
                         .short("p")
                         .long("print"))
                    .arg(Arg::with_name("bind-to")
                         .help("host:port to listen")
                         .required(false)));

    let matches = app.get_matches();

    let dictionary_path = path::get_dictionary_path()?;

    if let Some(ref matches) = matches.subcommand_matches("build") {
        let source_path = matches.value_of("dictionary-path").unwrap(); // Required
        command::builder::build_dictionary(&source_path, &dictionary_path)
    } else if matches.subcommand_matches("color").is_some() {
        command::terminal::color()
    } else if let Some(ref matches) = matches.subcommand_matches("lookup") {
        let word = matches.value_of("word").unwrap(); // Required
        command::terminal::lookup(&dictionary_path, word)
    } else if let Some(ref matches) = matches.subcommand_matches("server") {
        let bind_to = matches.value_of("bind-to").unwrap_or("127.0.0.1:8116");
        command::http::start_server(
            bind_to,
            HttpConfig {
                clear_before_print: matches.is_present("clear"),
                dictionary_path,
                do_print: matches.is_present("print"),
                ignore_not_found: matches.is_present("ignore"),
            })?;
        Ok(())
    } else {
        command::terminal::shell(&dictionary_path)
    }
}

fn main() {
    use failure::Fail;

    if let Err(err) = _main() {
        let mut fail: &Fail = &err;
        let mut message = err.to_string();

        while let Some(cause) = fail.cause() {
            message.push_str(&format!("\n\tcaused by: {}", cause));
            fail = cause;
        }

        eprintln!("Error: {}", message);

        exit(1);
    }
}
