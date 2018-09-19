
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



pub struct Config {
    pub clear_before_print: bool,
    pub dictionary_path: PathBuf,
    pub do_print: bool,
    pub ignore_not_found: bool,
}


pub fn start_server(bind_to: &str, config: Config) -> Result<(), AppError> {
    let mut server = Nickel::with_data(config);
    server.get("/ack", on_ack);
    server.get("/word/:word", on_get_word);
    server.listen(bind_to)?;
    Ok(())
}

fn set_cors<'mw>(response: &mut Response<'mw, Config>) {
    let headers = response.headers_mut();
    headers.set(AccessControlAllowOrigin::Any);
    headers.set(AccessControlAllowMethods(vec![Method::Get]));
    headers.set(AccessControlAllowHeaders(vec![UniCase("Content-Type".to_owned())]));
}

fn on_ack<'mw>(_: &mut Request<Config>, mut response: Response<'mw, Config>) -> MiddlewareResult<'mw, Config> {
    set_cors(&mut response);
    response.send("␆")
}

fn on_get_word<'mw>(request: &mut Request<Config>, mut response: Response<'mw, Config>) -> MiddlewareResult<'mw, Config> {
    set_cors(&mut response);

    let config = &*request.server_data();
    match get_word(&config.dictionary_path, request.param("word")) {
        Ok(entries) => {
            if config.do_print {
                if config.clear_before_print && (!config.ignore_not_found || entries.is_some()){
                    print!("\x1b[2J\x1b[H");
                }
                if let Err(err) = print_colored_opt(&entries, config.ignore_not_found) {
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
