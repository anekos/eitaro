
#[macro_use] extern crate clap;
#[macro_use] extern crate failure_derive;
#[macro_use] extern crate if_let_return;
extern crate app_dirs;
extern crate colored;
extern crate encoding;
extern crate failure;
extern crate kv;
extern crate nickel;
extern crate percent_encoding;
extern crate pom;
extern crate readline;

use std::fs::{create_dir_all, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::exit;

use app_dirs::{AppInfo, AppDataType};
use clap::{Arg, SubCommand};
use encoding::DecoderTrap::Replace;
use encoding::Encoding;
use encoding::all::WINDOWS_31J;

mod dictionary;
mod errors;
mod http;
mod loader;
mod printer;
mod str_utils;

use errors::AppError;
use loader::Loader;
use dictionary::Dictionary;



const APP_INFO: AppInfo = AppInfo { name: "eitaro", author: "anekos" };


fn get_dictionary_path() -> Result<PathBuf, AppError> {
    let result = app_dirs::get_app_dir(AppDataType::UserCache, &APP_INFO, "dictionary")?;
    if !result.exists() {
        create_dir_all(&result)?;
    }
    Ok(result)
}

fn build_dictionary<T: AsRef<Path>, U: AsRef<Path>>(source_path: &T, dictionary_path: &U) -> Result<(), AppError> {
    eprintln!("Building...");
    let mut buffer = vec![];
    let mut file = File::open(source_path)?;
    eprintln!("Reading...");
    let _ = file.read_to_end(&mut buffer)?;
    eprintln!("Encoding...");
    let decoded = WINDOWS_31J.decode(&buffer, Replace).map_err(|err| err.to_string())?;
    eprintln!("Loading...");
    let ldr = loader::eijiro::EijiroLoader::default();
    ldr.load(&decoded, dictionary_path).unwrap();
    Ok(())
}

fn lookup<T: AsRef<Path>>(dictionary_path: &T, word: &str) -> Result<(), AppError> {
    let mut dic = Dictionary::new(dictionary_path);
    lookup_and_print_lines(&mut dic, word)
}

fn interactive<T: AsRef<Path>>(dictionary_path: &T) -> Result<(), AppError> {
    use readline::Error::EndOfFile;

    let mut dic = Dictionary::new(dictionary_path);
    loop {
        match readline::readline("Eitaro> ") {
            Ok(ref input) => {
                readline::add_history(input)?;
                lookup_and_print_lines(&mut dic, input)?;
            },
            Err(EndOfFile) => {
                println!();
                break;
            },
            Err(_) => continue,
        }
    }

    Ok(())
}

fn lookup_and_print_lines(dic: &mut Dictionary, s: &str) -> Result<(), AppError> {
    for line in s.lines() {
        printer::print_colored_opt(dic.get(line.trim())?.as_ref().map(String::as_str))?;
    }
    Ok(())
}

fn _main() -> Result<(), AppError> {
    let app = app_from_crate!()
        .subcommand(SubCommand::with_name("lookup")
                    .alias("l")
                    .about("Lookup")
                    .arg(Arg::with_name("word")
                         .help("Word")
                         .required(true)))
        .subcommand(SubCommand::with_name("build")
                    .alias("b")
                    .about("Build dictionary")
                    .arg(Arg::with_name("dictionary-path")
                         .help("Dictionary file path")
                         .required(true)))
        .subcommand(SubCommand::with_name("server")
                    .alias("s")
                    .about("HTTP Server")
                    .arg(Arg::with_name("bind-to")
                         .help("host:port to listen")
                         .required(false)));

    let matches = app.get_matches();

    let dictionary_path = get_dictionary_path()?;

    if let Some(ref matches) = matches.subcommand_matches("build") {
        let source_path = matches.value_of("dictionary-path").unwrap(); // Required
        build_dictionary(&source_path, &dictionary_path)
    } else if let Some(ref matches) = matches.subcommand_matches("server") {
        let bind_to = matches.value_of("bind-to").unwrap_or("127.0.0.1:8116");
        http::main(&dictionary_path, bind_to)?;
        Ok(())
    } else if let Some(ref matches) = matches.subcommand_matches("lookup") {
        let word = matches.value_of("word").unwrap(); // Required
        lookup(&dictionary_path, word)
    } else {
        interactive(&dictionary_path)
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
