
use std::path::{Path, PathBuf};

use actix_cors::Cors;
use actix_web::{App,  HttpServer, Responder, web, http::header};
use serde_derive::*;

use crate::dictionary::{Dictionary, Entry};
use crate::errors::AppError;
use crate::screen::Screen;



#[derive(Clone)]
pub struct Config {
    pub curses: bool,
    pub dictionary_path: PathBuf,
    pub do_print: bool,
    pub ignore_not_found: bool,
    pub kuru: bool,
}

#[derive(Clone)]
pub struct State {
    config: Config,
    pub screen: Screen,
}

#[derive(Deserialize)]
pub struct GetWord {
    word: String,
}

pub fn start_server(bind_to: &str, mut config: Config) -> Result<(), AppError> {
    if config.kuru {
        config.curses = true;
    }
    if config.curses {
        config.do_print = true;
    }

    let screen = Screen::new(config.curses, config.kuru, bind_to.to_owned());
    let state = State { config, screen };

    // let mut server = Nickel::with_data(state);
    // server.get("/ack", on_ack);
    // server.get("/word/:word", on_get_word);
    // server.options = nickel::Options::default().output_on_listen(output_on_listen);
    // server.listen(bind_to)?;

    let server = HttpServer::new(move || {
        let state= state.clone();
        App::new()
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:8080")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .route("/ack", web::get().to(on_ack))
            .route("/word/{word}", web::get().to(on_get_word))
            .data(state)
    });

    server
        .bind(bind_to)?
        .run()?;

    Ok(())
}

fn on_ack() -> impl Responder {
    "␆"
}

fn on_get_word(state: web::Data<State>, param: web::Path<GetWord>) -> impl Responder {
    match get_word(&state.config.dictionary_path, &param.word) {
        Ok(entries) => {
            if state.config.do_print && (!state.config.ignore_not_found || entries.is_some()) {
                state.screen.print_opt(entries.clone());
            }
            if let Some(entries) = entries {
                let mut content = vec![];
                for entry in entries {
                    content.push(format!("#{}", entry.key));
                    // content.push(entry.content);
                }
                Some(content.join("\n"))
            } else {
                None
            }
        },
        Err(err) => panic!("Not implemented: {}", err)
    }
}

fn get_word<T: AsRef<Path>>(dictionary_path: &T, word: &str) -> Result<Option<Vec<Entry>>, AppError> {
    let mut dic = Dictionary::new(dictionary_path);
    Ok(dic.get_smart(&word)?)
}
