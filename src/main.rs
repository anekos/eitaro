
use std::path::Path;
use std::process::exit;

#[macro_use]
extern crate diesel;

use failure::Fail;
use structopt::StructOpt;
use structopt::clap::AppSettings;

#[macro_use] mod db;
mod command;
mod correction;
mod delay;
mod dictionary;
mod errors;
mod loader;
mod pager;
mod parser;
mod path;
mod screen;
mod str_utils;
mod types;

use crate::errors::{AppError, AppResultU};



#[derive(StructOpt, Debug)]
#[structopt(name = "eitaro")]
pub struct Opt {
    #[structopt(subcommand)]
    pub command: Option<Command>,
}

#[derive(StructOpt, Debug)]
#[structopt(setting = AppSettings::InferSubcommands)]
pub enum Command {
    /// Analyze text (STDIN) using SVL
    Analyze(command::analyze::Opt),
    /// Build dictionary
    Build(command::builder::Opt),
    /// Generate completions script for this command
    Completions(command::completions::Opt),
    /// Access dictionary database using sqlite
    #[structopt(alias = "db")]
    Database(command::database::Opt),
    /// Export the definitions for the given words (STDIN)
    Export(command::export::Opt),
    /// Output HTML fragment
    Html(command::html::Opt),
    /// Output keys
    Lemmas(command::lemmas::Opt),
    /// Lemmatize
    Lemmatize(command::lemmatize::Opt),
    /// Get word level (SVL)
    #[structopt(alias = "lv")]
    Level(command::level::Opt),
    /// Like
    Like(command::lookup::LikeOpt),
    /// Lookup
    Lookup(command::lookup::LookupOpt),
    /// Display the file paths using by eitaro
    Path,
    /// HTTP Server
    Server(command::http::Opt),
    /// Interactive shell
    Shell(command::lookup::ShellOpt),
    /// Untypo
    Untypo(command::untypo::Opt),
    /// Play wordle
    Wordle(command::wordle::Opt),
    /// Extract lemmatized words
    Words(command::words::Opt),
}




fn _main<T: AsRef<Path>>(dictionary_path: &T) -> AppResultU {
    use self::Command::*;

    let opt = Opt::from_args();

    if let Some(command) = opt.command {
        match command {
            Analyze(opt) =>
                command::analyze::analyze(opt, dictionary_path),
            Build(opt) =>
                command::builder::build_dictionary(opt, dictionary_path),
            Completions(opt) =>
                command::completions::generate(opt, Opt::clap()),
            Database(opt) =>
                command::database::shell(opt, dictionary_path),
            Export(opt) =>
                command::export::export(opt, dictionary_path),
            Html(opt) =>
                command::html::lookup(opt, dictionary_path),
            Lemmas(opt) =>
                command::lemmas::lemmas(opt, dictionary_path),
            Shell(opt) =>
                command::lookup::shell(opt, dictionary_path),
            Lemmatize(opt) =>
                command::lemmatize::lemmatize(opt, dictionary_path),
            Level(opt) =>
                command::level::level(opt, dictionary_path),
            Like(opt) =>
                command::lookup::like(opt, dictionary_path),
            Lookup(opt) =>
                command::lookup::lookup(opt, dictionary_path),
            Path =>
                command::path::path(dictionary_path),
            Server(opt) =>
                command::http::start_server(opt, dictionary_path.as_ref().to_path_buf()),
            Untypo(opt) =>
                command::untypo::untypo(opt, dictionary_path),
            Wordle(opt) =>
                command::wordle::play(opt, dictionary_path),
            Words(opt) =>
                command::words::extract(opt, dictionary_path),
        }
    } else if let Some(Command::Shell(opt)) = Opt::from_iter(&["", "shell"]).command {
        command::lookup::shell(opt, dictionary_path)
    } else {
        panic!("WTF: {:?}", Opt::from_iter(&["shell"]))
    }
}

fn main() {
    // Supress `failed printing to stdout: Broken pipe (os error 32)`
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }


    let dictionary_path = path::get_dictionary_path().expect("Failed to get dictionary path");

    match _main(&dictionary_path) {
        Err(AppError::Void) | Ok(_) => (),
        Err(err) => {
            if let AppError::Diesel(_) = err {
                eprintln!("Please build dictionary before use. See `eitaro build --help`");
                eprintln!("");
            }
            print_error(&err);
        }
    }
}

fn print_error(mut fail: &dyn Fail) {
    let mut message = fail.to_string();

    while let Some(cause) = fail.cause() {
        message.push_str(&format!("\n\tcaused by: {}", cause));
        fail = cause;
    }

    eprintln!("Error: {}", message);

    exit(1);
}
