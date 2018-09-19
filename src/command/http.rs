
use std::path::{Path, PathBuf};

use hyper::method::Method;
use hyper::header::{AccessControlAllowHeaders, AccessControlAllowMethods, AccessControlAllowOrigin};
use unicase::UniCase;
use nickel::status::StatusCode;
use nickel::{Nickel, HttpRouter, Request, Response, MiddlewareResult};
use percent_encoding::percent_decode;

use dictionary::{Dictionary, Entry};
use errors::{AppError, ErrorKind};
use printer::print_colored_opt;



struct Config {
    clear_before_print: bool,
    do_print: bool,
    path: PathBuf,
}


pub fn start_server<T: AsRef<Path>>(dictionary_path: &T, bind_to: &str, do_print: bool, clear_before_print: bool) -> Result<(), AppError> {
    let path: PathBuf = dictionary_path.as_ref().to_path_buf();
    let mut server = Nickel::with_data(Config { do_print, clear_before_print, path});

    server.get("/word/:word", on_get_word);
    server.listen(bind_to)?;
    Ok(())
}

fn on_get_word<'mw>(request: &mut Request<Config>, mut response: Response<'mw, Config>) -> MiddlewareResult<'mw, Config> {
    {
        let headers = response.headers_mut();
        headers.set(AccessControlAllowOrigin::Any);
        headers.set(AccessControlAllowMethods(vec![Method::Get]));
        headers.set(AccessControlAllowHeaders(vec![UniCase("Content-Type".to_owned())]));
    }

    let config = &*request.server_data();
    match get_word(&config.path, request.param("word")) {
        Ok(entries) => {
            if config.do_print {
                if config.clear_before_print {
                    print!("\x1b[2J\x1b[H");
                }
                if let Err(err) = print_colored_opt(&entries) {
                    eprintln!("Error: {}", err);
                }
            }
            if let Some(entries) = entries {
                let mut content = vec![];
                for entry in entries {
                    content.push(format!("#{}", entry.key));
                    content.push(entry.content);
                }
                response.send(content.join("\n"))
            } else {
                response.set(StatusCode::NotFound);
                response.send("Not found")
            }
        },
        Err(err) => response.send(format!("Error: {}", err)),
    }
}

fn get_word<T: AsRef<Path>>(dictionary_path: &T, word: Option<&str>) -> Result<Option<Vec<Entry>>, AppError> {
    let word = word.ok_or(ErrorKind::Eitaro("No `word` paramter"))?;
    let word = percent_decode(word.as_bytes()).decode_utf8()?;
    let mut dic = Dictionary::new(dictionary_path);
    Ok(dic.get(&word)?)
}
