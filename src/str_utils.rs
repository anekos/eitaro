

pub fn scan_words(s: &str) -> Vec<&str> {
    let mut result = vec![];
    let mut in_word = false;
    let mut index = 0;
    let mut left = 0;

    for c in s.chars() {
        if in_word ^ is_word_char(c) {
            in_word = !in_word;
            if in_word {
                left = index;
            } else {
                result.push(&s[left..index]);
            }
        }

        index += c.len_utf8();
    }

    if in_word {
        result.push(&s[left..index]);
    }

    result
}

fn is_word_char(c: char) -> bool {
    c.is_ascii() && c.is_alphanumeric() || c == '-'
}


#[cfg(test)]#[test]
fn test_scan_words() {
    assert_eq!(scan_words(" foo キャット bar 猫"), vec!["foo", "bar"]);
    assert_eq!(scan_words(" foo、キャット・bar=猫  "), vec!["foo", "bar"]);
    assert_eq!(scan_words(" foo-bar "), vec!["foo-bar"]);
    assert_eq!(scan_words("【変化】動 drives | driving | drove | driven"), vec!["drives", "driving", "drove", "driven"]);
    // TODO
    // drivels | drivel(l)ing | drivel(l)ed
    // 【変化】複 wildcats、【分節】wild・cat
}
