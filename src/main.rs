
use std::process::exit;

#[macro_use]
extern crate diesel;

use structopt::StructOpt;
use structopt::clap::AppSettings;

#[macro_use] mod db;
mod command;
mod correction;
mod delay;
mod dictionary;
mod errors;
mod loader;
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
    Database(command::database::Opt),
    /// Export the definitions for the given words (STDIN)
    Export(command::export::Opt),
    /// Output HTML fragment
    Html(command::html::Opt),
    /// Lemmatize
    Lemmatize(command::lemmatize::Opt),
    /// Get word level (SVL)
    Level(command::level::Opt),
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
    /// Extract lemmatized words
    Words(command::words::Opt),
}




fn _main() -> AppResultU {
    use self::Command::*;

    let dictionary_path = path::get_dictionary_path()?;

    let opt = Opt::from_args();

    if let Some(command) = opt.command {
        match command {
            Analyze(opt) =>
                command::analyze::analyze(opt, &dictionary_path),
            Build(opt) =>
                command::builder::build_dictionary(opt, &dictionary_path),
            Completions(opt) =>
                command::completions::generate(opt, Opt::clap()),
            Database(opt) =>
                command::database::shell(opt, &dictionary_path),
            Export(opt) =>
                command::export::export(opt, &dictionary_path),
            Html(opt) =>
                command::html::lookup(opt, &dictionary_path),
            Shell(opt) =>
                command::lookup::shell(opt, &dictionary_path),
            Lemmatize(opt) =>
                command::lemmatize::lemmatize(opt, &dictionary_path),
            Level(opt) =>
                command::level::level(opt, &dictionary_path),
            Lookup(opt) =>
                command::lookup::lookup(opt, &dictionary_path),
            Path =>
                command::path::path(&dictionary_path),
            Server(opt) =>
                command::http::start_server(opt, dictionary_path),
            Untypo(opt) =>
                command::untypo::untypo(opt, &dictionary_path),
            Words(opt) =>
                command::words::extract(opt, &dictionary_path),
        }
    } else {
        command::lookup::shell(command::lookup::ShellOpt::default(), &dictionary_path)
    }
}

fn main() {
    use failure::Fail;

    // Supress `failed printing to stdout: Broken pipe (os error 32)`
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

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
