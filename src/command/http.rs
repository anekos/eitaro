
use std::path::{Path, PathBuf};

use actix_cors::Cors;
use actix_web::{App,  HttpServer, Responder, web, http::header};
use serde_derive::*;

use crate::dictionary::{Dictionary, Entry};
use crate::errors::AppError;
use crate::screen::{Screen, ScreenConfig};



pub struct Config {
    pub dictionary_path: PathBuf,
    pub ignore_not_found: bool,
    pub screen: Option<ScreenConfig>,
}

#[derive(Clone)]
struct State {
    pub dictionary_path: PathBuf,
    pub ignore_not_found: bool,
    pub screen: Option<Screen>,
}

#[derive(Deserialize)]
pub struct GetWord {
    word: String,
}

pub fn start_server(bind_to: &str, config: Config) -> Result<(), AppError> {
    let state = State {
        dictionary_path: config.dictionary_path,
        ignore_not_found: config.ignore_not_found,
        screen: config.screen.map(|config| Screen::new(config, bind_to.to_owned()))
    };
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
    match get_word(&state.dictionary_path, &param.word) {
        Ok(entries) => {
            if let Some(screen) = &state.screen {
                if !state.ignore_not_found || entries.is_some() {
                    screen.print_opt(entries.clone());
                }
            }
            if let Some(entries) = entries {
                let mut content = vec![];
                for entry in &entries {
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
