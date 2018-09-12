
use std::path::{Path, PathBuf};
use std::error::Error;

use store::Dictionary;
use nickel::{Nickel, HttpRouter, Request, Response, MiddlewareResult};
use percent_encoding::{percent_decode};



pub fn main<T: AsRef<Path>>(dictionary_path: &T, bind_to: &str) -> Result<(), Box<Error>> {
    let path: PathBuf = dictionary_path.as_ref().to_path_buf();
    let mut server = Nickel::with_data(path);

    server.get("/word/:word", on_get_word);
    server.listen(bind_to)?;
    Ok(())
}

fn on_get_word<'mw>(request: &mut Request<PathBuf>, response: Response<'mw, PathBuf>) -> MiddlewareResult<'mw, PathBuf> {
    let path = &*request.server_data();
    match get_word(path, request.param("word")) {
        Ok(content) => response.send(content),
        Err(err) => response.send(format!("Error: {}", err)),
    }
}

fn get_word<T: AsRef<Path>>(dictionary_path: &T, word: Option<&str>) -> Result<String, Box<Error>> {
    let word = word.ok_or("No `word` paramter")?;
    let word = percent_decode(word.as_bytes()).decode_utf8()?.to_string();
    println!("on_get_word: {:?}", word);
    let mut dic = Dictionary::new(dictionary_path);
    Ok(dic.get(word).unwrap()) // FIXME
}
