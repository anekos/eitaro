
use std::path::{Path, PathBuf};

use hyper::header::{AccessControlAllowHeaders, AccessControlAllowMethods, AccessControlAllowOrigin};
use hyper::method::Method;
use nickel::status::StatusCode;
use nickel::{HttpRouter, MiddlewareResult, Nickel, Request, Response, self};
use percent_encoding::percent_decode;
use unicase::UniCase;

use dictionary::{Dictionary, Entry};
use errors::{AppError, ErrorKind};
use screen::Screen;



pub struct Config {
    pub curses: bool,
    pub dictionary_path: PathBuf,
    pub do_print: bool,
    pub ignore_not_found: bool,
    pub kuru: bool,
}

pub struct State {
    config: Config,
    pub screen: Screen,
}


pub fn start_server(bind_to: &str, config: Config) -> Result<(), AppError> {
    let output_on_listen = !config.curses;

    let screen = Screen::new(config.curses, config.kuru, bind_to.to_owned());
    let state = State { config, screen };

    let mut server = Nickel::with_data(state);
    server.get("/ack", on_ack);
    server.get("/word/:word", on_get_word);
    server.options = nickel::Options::default().output_on_listen(output_on_listen);
    server.listen(bind_to)?;
    Ok(())
}

fn set_cors<'mw>(response: &mut Response<'mw, State>) {
    let headers = response.headers_mut();
    headers.set(AccessControlAllowOrigin::Any);
    headers.set(AccessControlAllowMethods(vec![Method::Get]));
    headers.set(AccessControlAllowHeaders(vec![UniCase("Content-Type".to_owned())]));
}

fn on_ack<'mw>(_: &mut Request<State>, mut response: Response<'mw, State>) -> MiddlewareResult<'mw, State> {
    set_cors(&mut response);
    response.send("␆")
}

fn on_get_word<'mw>(request: &mut Request<State>, mut response: Response<'mw, State>) -> MiddlewareResult<'mw, State> {
    set_cors(&mut response);

    let state = &*request.server_data();
    match get_word(&state.config.dictionary_path, request.param("word")) {
        Ok(entries) => {
            if state.config.do_print {
                if !state.config.ignore_not_found || entries.is_some() {
                    state.screen.print_opt(entries.clone());
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
    Ok(dic.get_smart(&word)?)
}
