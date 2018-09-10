
use dictionary::Dictionary;
use loader::Loader;



#[derive(Default)]
pub struct EijiroLoader();


impl Loader for EijiroLoader {
    fn load(&self, source: &str) -> Dictionary {
        for line in source.lines() {
            println!("line: {:?}", line);
        }

        Dictionary { words: 0 }
    }
}
