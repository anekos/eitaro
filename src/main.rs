
#[macro_use] extern crate clap;
#[macro_use] extern crate if_let_return;
extern crate app_dirs;
extern crate encoding;
extern crate kv;
extern crate nickel;
extern crate percent_encoding;

use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::exit;

use app_dirs::{AppInfo, AppDataType};
use clap::{Arg, SubCommand};
use encoding::DecoderTrap::Replace;
use encoding::Encoding;
use encoding::all::WINDOWS_31J;

mod http;
mod loader;
mod store;

use loader::Loader;
use store::Dictionary;



const APP_INFO: AppInfo = AppInfo { name: "eitaro", author: "anekos" };


fn get_dictionary_path() -> Result<PathBuf, Box<Error>> {
    let result = app_dirs::get_app_dir(AppDataType::UserCache, &APP_INFO, "dictionary")?;
    if !result.exists() {
        create_dir_all(&result).unwrap();
    }
    Ok(result)
}

fn build_dictionary<T: AsRef<Path>, U: AsRef<Path>>(source_path: &T, dictionary_path: &U) -> Result<(), Box<Error>> {
    eprintln!("Building...");
    let mut buffer = vec![];
    let mut file = File::open(source_path)?;
    eprintln!("Reading...");
    let _ = file.read_to_end(&mut buffer)?;
    eprintln!("Encoding...");
    let decoded = WINDOWS_31J.decode(&buffer, Replace)?;
    eprintln!("Loading...");
    let ldr = loader::eijiro::EijiroLoader::default();
    let _ = ldr.load(&decoded, dictionary_path);
    Ok(())
}

fn lookup<T: AsRef<Path>>(dictionary_path: &T, word: &str) -> Result<(), Box<Error>> {
    let mut dic = Dictionary::new(dictionary_path);
    println!("{}", dic.get(word.to_owned()).unwrap());
    Ok(())
}

fn _main() -> Result<(), Box<Error>> {
    let app = app_from_crate!()
        .subcommand(SubCommand::with_name("lookup")
                    .about("Lookup")
                    .arg(Arg::with_name("word")
                         .help("Word")
                         .required(true)))
        .subcommand(SubCommand::with_name("build")
                    .about("Build dictionary")
                    .arg(Arg::with_name("dictionary-path")
                         .help("Dictionary file path")
                         .required(true)))
        .subcommand(SubCommand::with_name("server")
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
        let bind_to = matches.value_of("bind-to").unwrap_or("127.0.0.1:6767");
        http::main(&dictionary_path, bind_to)
    } else if let Some(ref matches) = matches.subcommand_matches("lookup") {
        let word = matches.value_of("word").unwrap(); // Required
        lookup(&dictionary_path, word)
    } else {
        panic!("WTF!");
    }
}

fn main() {
    if let Err(err) =  _main() {
        eprintln!("Error: {:?}", err);
        eprintln!("Usage: eitaro <DICTIONARY>");
        exit(1);
    }
}
