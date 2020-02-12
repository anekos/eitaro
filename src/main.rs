
use std::process::exit;

use structopt::StructOpt;

mod args;
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



const DEFAULT_PROMPT: &str = "Eitaro> ";


#[derive(StructOpt, Debug)]
#[structopt(name = "server")]
pub struct Opt {
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    Analyze(command::analyze::Opt),
    Build(command::builder::Opt),
    Server(command::http::Opt),
}




fn _main() -> AppResultU {
    use self::Command::*;

    let dictionary_path = path::get_dictionary_path()?;

    let opt = Opt::from_args();
    println!("{:?}", opt);

    match opt.command {
        Analyze(opt) =>
            command::analyze::analyze(opt, &dictionary_path)?,
        Build(opt) =>
            command::builder::build_dictionary(opt, &dictionary_path)?,
        Server(opt) =>
            command::http::start_server(opt, dictionary_path)?
    }

    Ok(())

    // let app = args::build();
    //
    // let matches = app.get_matches();
    //
    // let dictionary_path = path::get_dictionary_path()?;
    //
    // if let Some(ref matches) = matches.subcommand_matches("analyze") {
    //     let all = matches.is_present("all");
    //     let usage = matches.value_of("usage").map(|it| it.parse());
    //     let target = command::analyze::Target {
    //         count: all || matches.is_present("count"),
    //         not_in_dictionary: all || matches.is_present("not-in-dictionary"),
    //         out_of_level: all || matches.is_present("out-of-level"),
    //         svl: all || matches.is_present("svl"),
    //         usage: matches.is_present("usage"),
    //     };
    //     command::analyze::analyze(&dictionary_path, target)
    // } else if let Some(ref matches) = matches.subcommand_matches("build") {
    //     let files: Vec<&str> = matches.values_of("dictionary-file").unwrap().collect(); // Required
    //     command::builder::build_dictionary(&files, &dictionary_path)
    // } else if let Some(ref matches) = matches.subcommand_matches("export") {
    //     let as_text = matches.is_present("as-text");
    //     command::export::export(&dictionary_path, as_text)
    // } else if let Some(ref matches) = matches.subcommand_matches("html") {
    //     let word = matches.value_of("word").unwrap(); // Required
    //     command::html::lookup(&dictionary_path, word)
    // } else if let Some(ref matches) = matches.subcommand_matches("interactive") {
    //     command::lookup::shell(&dictionary_path, matches.value_of("prompt").unwrap_or(DEFAULT_PROMPT))
    // } else if let Some(ref matches) = matches.subcommand_matches("lemmatize") {
    //     let word = matches.value_of("word").unwrap(); // Required
    //     command::lemmatize::lemmatize(&dictionary_path, word)
    // } else if let Some(ref matches) = matches.subcommand_matches("level") {
    //     let word = matches.value_of("word").unwrap(); // Required
    //     command::level::level(&dictionary_path, word)
    // } else if let Some(ref matches) = matches.subcommand_matches("lookup") {
    //     let word = matches.value_of("word").unwrap(); // Required
    //     let n = matches.value_of("n").map(|it| it.parse()).transpose()?;
    //     let color = !matches.is_present("no-color");
    //     command::lookup::lookup(&dictionary_path, word, color, n)
    // } else if matches.subcommand_matches("path").is_some() {
    //     command::path::path(&dictionary_path)
    // } else if let Some(ref matches) = matches.subcommand_matches("server") {
    //     let bind_to = matches.value_of("bind-to").unwrap_or("127.0.0.1:8116");
    //     let kuru = matches.is_present("kuru");
    //     let screen = if matches.is_present("curses") || kuru {
    //         Some(ScreenConfig::Curses { kuru })
    //     } else if matches.is_present("print") {
    //         Some(ScreenConfig::Color)
    //     } else if matches.is_present("gui") {
    //         let font_name: Option<String> = matches.value_of("font-name").map(ToOwned::to_owned);
    //         let font_size: f64 = matches.value_of("font-size").unwrap().parse()?; // Default
    //         let config = screen::gui::Config { dictionary_path: dictionary_path.clone(), font_name, font_size };
    //         Some(ScreenConfig::Gui(config))
    //     } else if matches.is_present("plain") {
    //         Some(ScreenConfig::Plain)
    //     } else {
    //         None
    //     };
    //     command::http::start_server(
    //         bind_to,
    //         HttpConfig {
    //             dictionary_path,
    //             ignore_not_found: matches.is_present("ignore"),
    //             screen,
    //         })?;
    //     Ok(())
    // } else if let Some(ref matches) = matches.subcommand_matches("untypo") {
    //     let word = matches.value_of("word").unwrap(); // Required
    //     command::untypo::untypo(&dictionary_path, word)
    // } else {
    //     command::lookup::shell(&dictionary_path, DEFAULT_PROMPT)
    // }
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
