
use kana::wide2ascii;



#[derive(Clone, Copy)]
pub enum WordType {
    English,
    Katakana,
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

pub fn scan_words(word_type: WordType, s: &str) -> Vec<&str> {
    let mut result = vec![];
    let mut in_word = false;
    let mut index = 0;
    let mut left = 0;
    let is_word_char = get_is_word_char(word_type);

    for c in s.chars() {
        if in_word ^ is_word_char(c) {
            in_word = !in_word;
            if in_word {
                left = index;
            } else if left < index {
                result.push(&s[left..index]);
            }
        }

        index += c.len_utf8();
    }

    if in_word && left < index {
        result.push(&s[left..index]);
    }

    result
}

fn get_is_word_char(word_type: WordType) -> fn(char) -> bool {
    match word_type {
        WordType::English => is_word_char_english,
        WordType::Katakana => is_word_char_katakana,
    }
}

fn is_word_char_english(c: char) -> bool {
    c.is_ascii() && c.is_alphanumeric() || c == '-' || c == '\''
}

// FIXME
fn is_word_char_katakana(c: char) -> bool {
    !c.is_ascii() && c.is_alphabetic()
}


#[cfg(test)]#[test]
fn test_scan_words() {
    use  self::WordType::*;

    assert_eq!(scan_words(English, " foo キャット bar 猫"), vec!["foo", "bar"]);
    assert_eq!(scan_words(English, " foo、キャット・bar=猫  "), vec!["foo", "bar"]);
    assert_eq!(scan_words(English, " foo-bar "), vec!["foo-bar"]);
    assert_eq!(scan_words(English, "【変化】動 drives | driving | drove | driven"), vec!["drives", "driving", "drove", "driven"]);

    assert_eq!(scan_words(Katakana, "アカムパニ、アカンパニ、アコンパニ、"), vec!["アカムパニ", "アカンパニ", "アコンパニ"]);
    assert_eq!(scan_words(Katakana, " foo-bar "), Vec::<&str>::new());
    // FIXME
    // assert_eq!(scan_words(Katakana, " foo、キャット・bar=猫  "), vec!["キャット"]);

    // TODO
    // drivels | drivel(l)ing | drivel(l)ed
    // 【変化】複 wildcats、【分節】wild・cat
}
