#[derive(Queryable)]
pub struct Alias {
    pub id: i32,
    pub key: String,
    pub target: String,
}

#[derive(Queryable)]
pub struct Definition {
    pub id: i32,
    pub term: String,
    pub definition: String,
    pub text: String,
    pub source: Option<String>,
}

#[derive(Queryable)]
pub struct Lemmatization {
    pub id: i32,
    pub source: String,
    pub target: String,
}

