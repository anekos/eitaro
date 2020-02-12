
use std::path::PathBuf;

use actix_cors::Cors;
use actix_web::{App,  HttpServer, Responder, web, http::header};
use serde_derive::*;
use structopt::StructOpt;

use crate::dictionary::Dictionary;
use crate::errors::AppError;
use crate::screen::{Screen, Opt as ScreenOpt};



#[derive(StructOpt, Debug)]
#[structopt(name = "server")]
pub struct Opt {
    pub bind_to: Option<String>,
    #[structopt(short = "i", long = "ignore")]
    pub ignore_not_found: bool,
    #[structopt(subcommand)]
    pub screen: ScreenOpt,
}

#[derive(Clone)]
struct State {
    pub dictionary_path: PathBuf,
    pub ignore_not_found: bool,
    pub screen: Screen,
}

#[derive(Deserialize)]
pub struct GetWord {
    word: String,
}

pub fn start_server(opt: Opt, dictionary_path: PathBuf) -> Result<(), AppError> {
    let bind_to = opt.bind_to.unwrap_or_else(|| "127.0.0.1:8116".to_owned());
    let state = State {
        dictionary_path: dictionary_path.clone(),
        ignore_not_found: opt.ignore_not_found,
        screen: Screen::new(opt.screen, dictionary_path, bind_to.clone())
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
    match Dictionary::get_word(&state.dictionary_path, &param.word) {
        Ok(entries) => {
            if !state.ignore_not_found || entries.is_some() {
                state.screen.print_opt(entries.clone());
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
