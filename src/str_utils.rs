
use heck::SnakeCase;
use kana::wide2ascii;
use regex::Regex;



#[derive(Clone, Copy)]
pub enum WordType {
    English,
    Katakana,
}


pub fn simple_words_pattern() -> Regex {
    Regex::new(r"[a-zA-Z]+").unwrap()
}

pub fn fix_word(s: &str) -> Option<String> {
    let s = wide2ascii(s);
    let s = s.to_lowercase().replace('ー', "");
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

pub fn scan_words(word_type: WordType, s: &str) -> Vec<String> {
    let mut result = vec![];
    let mut in_word = false;
    let mut index = 0;
    let mut left = 0;
    let mut right = 0;
    let is_word_char = get_is_word_char(word_type);

    for c in s.chars() {
        let space = c == ' ';
        if in_word ^ (is_word_char(c) || (in_word && space)) {
            in_word = !in_word;
            if in_word {
                left = index;
            } else if left < right {
                extract_patterns(&s[left..right], &mut result);
            }
        }
        index += c.len_utf8();
        if in_word && !space {
            right = index;
        }
    }

    if in_word && left < right {
        extract_patterns(&s[left..right], &mut result);
    }

    result
}

pub fn shorten(s: &str) -> Vec<&str>  {
    let mut result = vec![];
    let mut index = 0;
    let mut left = 0;
    let mut in_word = false;
    let mut first = true;

    for c in s.chars() {
        if in_word ^ (c != ' ') {
            in_word = !in_word;
            if in_word {
                if first {
                    left = index;
                    first = false;
                }
            } else if left < index {
                result.push(&s[left..index]);
            }
        }

        index += c.len_utf8();
    }

    if in_word && left < index {
        result.push(&s[left..index]);
    }

    result.reverse();
    result
}

pub fn uncase(s: &str) -> String {
    s.to_snake_case().replace('_', " ")
}

fn extract_patterns(s: &str, result: &mut Vec<String>) {
    if let Some(l) = s.find('(') {
        if let Some(r) = s.find(')') {
            extract_patterns(&format!("{}{}", &s[0..l], &s[r+1..]), result);
            extract_patterns(&format!("{}{}{}", &s[0..l], &s[l+1..r], &s[r+1..]), result);
        } else {
            extract_patterns(&s[0..l].to_owned(), result);
        }
    } else {
        result.push(s.to_owned());
    }
}

fn get_is_word_char(word_type: WordType) -> fn(char) -> bool {
    match word_type {
        WordType::English => is_word_char_english,
        WordType::Katakana => is_word_char_katakana,
    }
}

fn is_word_char_english(c: char) -> bool {
    c.is_ascii() && c.is_alphanumeric() || c == '-' || c == '\'' || c == '(' || c == ')'
}

// FIXME
fn is_word_char_katakana(c: char) -> bool {
    !c.is_ascii() && c.is_alphabetic()
}



#[cfg(test)]#[test]
fn test_scan_words() {
    use  self::WordType::*;

    assert_eq!(scan_words(English, "  foo キャット  bar 猫"), vec!["foo", "bar"]);
    assert_eq!(scan_words(English, "  foo キャット  bar "), vec!["foo", "bar"]);
    assert_eq!(scan_words(English, " foo、キャット・bar=猫  "), vec!["foo", "bar"]);
    assert_eq!(scan_words(English, " foo-bar "), vec!["foo-bar"]);
    assert_eq!(scan_words(English, "【変化】動 drives | driving | drove | driven"), vec!["drives", "driving", "drove", "driven"]);
    assert_eq!(scan_words(English, "【変化】動 foo bar | food bar | foolish bar"), vec!["foo bar", "food bar", "foolish bar"]);
    assert_eq!(scan_words(English, "【変化】 複 affairs、【文節】..."), vec!["affairs"]);

    assert_eq!(scan_words(Katakana, "アカムパニ、アカンパニ、アコンパニ、"), vec!["アカムパニ", "アカンパニ", "アコンパニ"]);
    assert_eq!(scan_words(Katakana, " foo-bar "), Vec::<&str>::new());
    // FIXME
    // assert_eq!(scan_words(Katakana, " foo、キャット・bar=猫  "), vec!["キャット"]);

    assert_eq!(scan_words(English, " f(o)o キャット bar 猫"), vec!["fo", "foo", "bar"]);

    // TODO
    // drivels | drivel(l)ing | drivel(l)ed
    // 【変化】複 wildcats、【分節】wild・cat
}

#[cfg(test)]#[test]
fn test_patterns() {
    fn ps(s: &str) -> Vec<String> {
        let mut result = vec![];
        extract_patterns(s, &mut result);
        result
    }

    assert_eq!(ps("ana(a)l nathrakh"), vec!["anal nathrakh".to_owned(), "anaal nathrakh".to_owned()]);
    assert_eq!(
        ps("ab(c)de(f)g"),
        vec![
        "abdeg".to_owned(),
        "abdefg".to_owned(),
        "abcdeg".to_owned(),
        "abcdefg".to_owned()]);
}

#[cfg(test)]#[test]
fn test_shortens() {
    assert_eq!(
        shorten("the cat of hell"),
        vec![
        "the cat of hell".to_owned(),
        "the cat of".to_owned(),
        "the cat".to_owned(),
        "the".to_owned()
        ]);

    assert_eq!(
        shorten("   the cat of hell"),
        vec![
        "the cat of hell".to_owned(),
        "the cat of".to_owned(),
        "the cat".to_owned(),
        "the".to_owned()
        ]);

    assert_eq!(
        shorten(" the cat of hell    "),
        vec![
        "the cat of hell".to_owned(),
        "the cat of".to_owned(),
        "the cat".to_owned(),
        "the".to_owned()
        ]);

    assert_eq!(
        shorten(" the cat   of hell    "),
        vec![
        "the cat   of hell".to_owned(),
        "the cat   of".to_owned(),
        "the cat".to_owned(),
        "the".to_owned()
        ]);
}
