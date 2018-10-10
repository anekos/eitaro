

pub fn v2s(s: Vec<char>) -> String {
    let s: String = s.into_iter().collect();
    s.trim().to_owned()
}
