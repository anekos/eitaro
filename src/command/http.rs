
use std::path::{Path, PathBuf};

use actix_cors::Cors;
use actix_web::{App,  HttpServer, Responder, web, http::header};
use serde_derive::*;

use crate::dictionary::{Dictionary, Entry};
use crate::errors::AppError;
use crate::screen::{Screen, ScreenConfig};



#[derive(Clone)]
pub struct Config {
    pub color: bool,
    pub curses: bool,
    pub dictionary_path: PathBuf,
    pub gui: bool,
    pub ignore_not_found: bool,
    pub kuru: bool,
    pub plain: bool,
}

#[derive(Clone)]
pub struct State {
    config: Config,
    pub screen: Option<Screen>,
}

#[derive(Deserialize)]
pub struct GetWord {
    word: String,
}

pub fn start_server(bind_to: &str, config: Config) -> Result<(), AppError> {
    let screen = config.screen_config().map(|conf| Screen::new(conf, bind_to.to_owned()));
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
            if let Some(screen) = &state.screen {
                if !state.config.ignore_not_found || entries.is_some() {
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

impl Config {
    fn screen_config(&self) -> Option<ScreenConfig> {
        Some(
            if self.kuru || self.curses {
                ScreenConfig::Curses { kuru: self.kuru }
            } else if self.gui {
                ScreenConfig::Gui
            } else if self.color {
                ScreenConfig::Color
            } else if self.plain {
                ScreenConfig::Plain
            } else {
                return None
            }
        )
    }
}
