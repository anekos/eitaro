
use crate::dictionary::Text;


pub fn parse_line(input: &str) -> Result<Vec<Text>, String> {
    Ok(vec![Text::Definition(input.to_owned())])
}
