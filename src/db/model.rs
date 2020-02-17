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
}

#[derive(Queryable)]
pub struct Level {
    pub id: i32,
    pub term: String,
    pub level: i32,
}

#[derive(Queryable)]
pub struct Lemmatization {
    pub id: i32,
    pub source: String,
    pub target: String,
}

