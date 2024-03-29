
use std::collections::HashSet;
use std::default::Default;
use std::path::{Path, PathBuf};

use array_tool::vec::Uniq;
use diesel::connection::Connection;
use diesel::sqlite::SqliteConnection;
use if_let_return::if_let_some;
use indexmap::indexset;
use lazy_init::Lazy;
use regex::Regex;
use serde_derive::{Serialize, Deserialize};

use crate::correction::Corrector;
use crate::db::model::{Definition as ModelDef};
use crate::errors::{AppError, AppResult, AppResultU};
use crate::str_utils::{fix_word, shorten, uncase};



pub struct Dictionary  {
    corrector: Lazy<AppResult<Corrector>>,
    path: PathBuf,
} 

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Text {
    Annot(String),
    Class(String),
    Countability(char),
    Definition(String),
    Error(String),
    Etymology(String),
    Example(String),
    Information(String),
    Note(String),
    Tag(String),
    Word(String),
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Definition {
    pub key: String,
    pub content: Vec<Text>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Entry {
    pub key: String,
    pub definitions: Vec<Definition>,
}

#[derive(Clone)]
pub struct DictionaryWriter<'a> {
    connection: &'a SqliteConnection,
    source: Option<&'a str>,
}

pub struct Stat {
    pub aliases: usize,
    pub words: usize,
}



impl Dictionary {
    pub fn new<T: AsRef<Path>>(dictionary_path: &T) -> Self {
        Dictionary {
            corrector: Lazy::new(),
            path: dictionary_path.as_ref().to_path_buf()
        }
    }

    pub fn correct(&mut self, word: &str) -> Vec<String> {
        println!("Correcting...");

        let corrector = self.corrector.get_or_create(|| {
            let connection = self.connect_db()?;
            let keys = diesel_query!(definitions [Q R] {
                d::definitions
                    .select(d::term)
                    .load::<String>(&connection)?
            });
            Ok(Corrector { keys: keys.into_iter().collect() })
        });

        match corrector {
            Ok(corrector) => {
                corrector.correct(word)
            }
            Err(error) => {
                eprintln!("{}", error);
                vec![]
            }
        }
    }

    pub fn get_word<T: AsRef<Path>>(dictionary_path: &T, word: &str) -> Result<Option<Vec<Entry>>, AppError> {
        let mut dic = Dictionary::new(dictionary_path);
        Ok(dic.get_smart(&word)?)
    }

   pub fn get(&mut self, word: &str) -> AppResult<Option<Vec<Entry>>> {
        fn opt(result: Vec<Entry>) -> Option<Vec<Entry>> {
            if result.is_empty() {
                return None;
            }
            Some(result)
        }

        let connection = self.connect_db()?;

        let mut candidates = indexset!(word.to_owned());
        let mut result = vec![];

        for stemmed in stem(&word) {
            candidates.insert(stemmed);
        }

        if let Some(aliases) = lookup_unaliased(&connection, word)? {
            for alias in aliases.split('\n') {
                candidates.insert(alias.to_owned());
            }
        }

        for candidate in &candidates {
            if let Some(entry) = lookup_entry(&connection, candidate)? {
                result.push(entry);
            }
        }

        Ok(opt(result))
   }

   pub fn get_level(&mut self, word: &str) -> AppResult<Option<u8>> {
       fn get_level(connection: &SqliteConnection, word: &str) -> AppResult<Option<u8>> {
           diesel_query!(levels [Q E R O] {
               let found = d::levels
                   .filter(d::term.eq(word))
                   .select(d::level)
                   .first::<i32>(connection)
                   .optional()?;

               Ok(found.map(|it| it as u8))
           })
       }

       let connection = self.connect_db()?;

       let found = get_level(&connection, word)?;
       if found.is_some() {
           return Ok(found)
       }

       let lemmed = lemmatize(&connection, word)?;
       get_level(&connection, &lemmed)
   }

   pub fn get_smart(&mut self, word: &str) -> Result<Option<Vec<Entry>>, AppError> {
        if_let_some!(fixed = fix_word(word), Ok(None));

        for shortened in shorten(&fixed) {
            let mut result = self.get_similars(&shortened)?;
            if let Some(result) = result.as_mut() {
                return Ok(Some(result.unique()))
            }
        }

        let uncased = uncase(&word);
        if uncased != word {
            if let Some(result) = self.get_smart(&uncased)? {
                return Ok(Some(result))
            }
        }

        let splitter = Regex::new(r"[-#'=\s]+")?;
        let mut candidates: Vec<&str> = splitter.split(&fixed).collect();
        candidates.sort_by(|a, b| a.len().cmp(&b.len()).reverse());
        for candidate in candidates {
            let result = self.get(candidate)?;
            if result.is_some() {
                return Ok(result);
            }
        }

        Ok(None)
    }

    pub fn keys(&mut self) -> AppResult<Vec<String>> {
        let connection = self.connect_db()?;
        let keys = diesel_query!(definitions [Q R] {
            d::definitions
                .select(d::term)
                .load::<String>(&connection)?
        });

        Ok(keys)
    }

    pub fn lemmatize(&mut self, word: &str) -> AppResult<String> {
        let connection = self.connect_db()?;
        lemmatize(&connection, word)
    }

    pub fn like(&self, query: &str) -> AppResult<Option<Vec<Entry>>> {
        let connection = self.connect_db()?;

        let found: Vec<ModelDef> = diesel_query!(definitions, Definition [Q R T] {
            d::definitions.
                filter(d::term.like(query))
                .order((d::term, d::id))
                .load::<Definition>(&connection)?
        });

        if found.is_empty() {
            return Ok(None)
        }

        Ok(Some(compact_definitions(found)?))
    }

    pub fn search(&self, query: &str) -> AppResult<Option<Vec<Entry>>> {
        let connection = self.connect_db()?;

        let found = diesel_query!(definitions, Definition [B E Q R T] {
            use diesel::BoxableExpression;
            use diesel::sql_types::Bool;

            let truee = Box::new(d::term.eq(d::term));
            let q: Box<dyn BoxableExpression<d::definitions, _, SqlType = Bool>> =
                query.split_ascii_whitespace()
                .map(|it| d::text.like(format!("%{}%", it)))
                .fold(truee, |q, it| Box::new(q.and(it)));

            d::definitions.
                filter(q)
                .order((d::term, d::id))
                .load::<Definition>(&connection)?
        });

        Ok(Some(compact_definitions(found)?))
    }

    pub fn write<F>(&mut self, mut f: F) -> AppResult<Stat> where F: FnMut(&mut DictionaryWriter) -> AppResultU {
        if self.path.exists() {
            std::fs::remove_file(&self.path)?;
        }

        let connection = self.connect_db()?;

        diesel_query!([R] {
            for sql in include_str!("../migrations.sql").split(';') {
                diesel::sql_query(sql).execute(&connection)?;
            }
        });

        connection.transaction::<_, AppError, _>(|| {
            use crate::db::schema;
            use diesel::RunQueryDsl;

            diesel::delete(schema::aliases::dsl::aliases).execute(&connection)?;
            diesel::delete(schema::definitions::dsl::definitions).execute(&connection)?;
            diesel::delete(schema::lemmatizations::dsl::lemmatizations).execute(&connection)?;
            diesel::delete(schema::levels::dsl::levels).execute(&connection)?;

            let mut writer = DictionaryWriter::new(&connection, None);
            f(&mut writer)?;
            stat(&connection)
        })
    }

    fn connect_db(&self) -> AppResult<SqliteConnection> {
        let path = self.path.to_str().ok_or(AppError::Unexpect("WTF: connection"))?;
        Ok(SqliteConnection::establish(path)?)
    }

    fn get_similars(&mut self, word: &str) -> AppResult<Option<Vec<Entry>>> {
        let mut result = self.get(word)?;

        {
            let mut mutated = vec![];
            let chars = [',', '\'', '=', ' '];
            for from in &chars {
                for to in &["-", " ", ""] {
                    let replaced = word.replace(*from, to);
                    if replaced != word {
                        if let Some(result) = self.get(&replaced)? {
                            mutated.extend_from_slice(&result);
                        }
                    }
                }
            }

            if !mutated.is_empty() {
                if result.is_none() {
                    result = Some(mutated);
                } else if let Some(content) = result.as_mut() {
                    content.extend_from_slice(&mutated);
                }
            }
        }

        Ok(result)
    }

    pub fn wordle_words(&self, min: u8, max: u8) -> AppResult<Vec<String>> {
        let connection = self.connect_db()?;

        let found = diesel_query!(levels [E Q R T] {
            d::levels
                .filter(d::level.ge(i32::from(min)))
                .filter(d::level.le(i32::from(max)))
                .filter(d::term.like("_____"))
                .select(d::term)
                .load::<String>(&connection)?
        });

        Ok(found)
    }
}


fn compact_definitions(defs: Vec<ModelDef>) -> AppResult<Vec<Entry>> {
    let defs: serde_json::Result<Vec<(String, Definition)>> =
        defs.into_iter().map(|it| serde_json::from_str::<Definition>(&it.definition).map(|d| (it.term, d))).collect();
    let defs = defs?;

    let mut result = vec![];
    let mut buffer = vec![];
    let mut last_key = defs[0].0.clone();

    for (key, def) in defs {
        if key == last_key {
            buffer.push(def);
        } else {
            let mut definitions = vec![def];
            let mut key = key;
            std::mem::swap(&mut buffer, &mut definitions);
            std::mem::swap(&mut key, &mut last_key);
            result.push(Entry { key, definitions });
        }
    }

    if !buffer.is_empty() {
        result.push(Entry { key: last_key, definitions: buffer });
    }

    Ok(result)
}

fn lemmatize(connection: &SqliteConnection, word: &str) -> AppResult<String> {
    let mut lemmed = word.to_owned();
    let mut path = HashSet::<String>::new();

    while let Some(found) = lookup_lemmatized(connection, &lemmed)? {
        if !path.insert(found.clone()) {
            return Ok(lemmed)
        }
        lemmed = found;
    }

    if lookup_entry(connection, &lemmed)?.is_some() {
        return Ok(lemmed.to_owned());
    }

    for stemmed in stem(&lemmed) {
        if lookup_entry(connection, &stemmed)?.is_some() {
            return Ok(stemmed);
        }
    }

    Ok(lemmed.to_owned())
}

fn lookup_entry(connection: &SqliteConnection, word: &str) -> AppResult<Option<Entry>> {
    let found = diesel_query!(definitions, Definition [Q E R] {
        d::definitions
            .filter(d::term.eq(word))
            .load::<Definition>(connection)?
    });

    if found.is_empty() {
        return Ok(None)
    }

    let defs: serde_json::Result<Vec<Definition>> = found.iter().map(|it| serde_json::from_str::<Definition>(&it.definition)).collect();

    Ok(Some(Entry {
        key: word.to_owned(),
        definitions: defs?,
    }))
}

fn lookup_lemmatized(connection: &SqliteConnection, word: &str) -> AppResult<Option<String>> {
    diesel_query!(lemmatizations, Lemmatization [Q E R] {
        let found = d::lemmatizations
            .filter(d::source.eq(word))
            .limit(1)
            .load::<Lemmatization>(connection)?;

        Ok(found.get(0).map(|it| it.target.to_owned()))
    })
}

fn lookup_unaliased(connection: &SqliteConnection, word: &str) -> AppResult<Option<String>> {
    diesel_query!(aliases, Alias [Q E R] {
        let found = d::aliases
            .filter(d::source.eq(word))
            .limit(1)
            .load::<Alias>(connection)?;

        Ok(found.get(0).map(|it| it.target.to_owned()))
    })
}

fn stat(connection: &SqliteConnection) -> AppResult<Stat> {
    // FIXME
    let words = diesel_query!(definitions [Q R] {
        d::definitions
            .select(d::term)
            .distinct()
            .load::<String>(connection)
    })?.len();
    let aliases = diesel_query!(aliases [Q R] {
        use diesel::dsl::count;
        d::aliases
            .select(count(d::id))
            .first::<i64>(connection)
    })? as usize;

    Ok(Stat { aliases, words })
}

fn stem(word: &str) -> Vec<String> {
    let pairs = [
        ("ied", "y"),
        ("ier", "y"),
        ("ies", "y"),
        ("iest", "y"),
        ("nning", "n"),
        ("est", ""),
        ("ing", ""),
        ("'s", ""),
        ("ed", ""),
        ("ed", "e"),
        ("er", ""),
        ("es", ""),
        ("s", ""),
    ];

    let mut result = vec![];
    let wlen = word.len();

    for (suffix, to) in &pairs {
        if wlen < suffix.len() + 2 {
            break;
        }

        if word.ends_with(suffix) {
            result.push(format!("{}{}", &word[0 .. wlen - suffix.len()], to));
        }
    }

    result
}



impl<'a> DictionaryWriter<'a> {
    fn new(connection: &'a SqliteConnection, source: Option<&'a str>) -> Self {
        DictionaryWriter {
            connection,
            source
        }
    }

    pub fn with_source(self, source: Option<&'a str>) -> Self {
        DictionaryWriter {
            connection: self.connection,
            source,
        }
    }

    pub fn alias(&mut self, from: &str, to: &str, for_lemmatization: bool) -> AppResultU {
        if let (Some(from), Some(to)) = (fix_word(from), fix_word(to)) {
            if from == to {
                return Ok(());
            }

            if for_lemmatization {
                diesel_query!(lemmatizations [E R] {
                    diesel::insert_into(d::lemmatizations)
                        .values((d::source.eq(&from), d::target.eq(&to)))
                        .execute(self.connection)?;
                });
            }

            diesel_query!(aliases [E R] {
                diesel::insert_into(d::aliases)
                    .values((d::source.eq(&from), d::target.eq(&to)))
                    .execute(self.connection)?;
            });
        }
        Ok(())
    }

    pub fn define(&mut self, key: &str, content: Vec<Text>) -> AppResultU {
        let lkey = key.to_lowercase();

        let mut buffer = "".to_owned();
        for it in &content {
            if let Some(s) = it.text_for_search() {
                if !buffer.is_empty() {
                    buffer.push(' ');
                }
                buffer.push_str(s);
            }
        }

        let def = Definition { key: key.to_owned(), content };

        diesel_query!(definitions [E R] {
            let serialized = serde_json::to_string(&def).unwrap();
            diesel::insert_into(d::definitions)
                .values((d::term.eq(lkey), d::definition.eq(serialized), d::text.eq(&buffer), d::source.eq(self.source)))
                .execute(self.connection)?;
        });

        Ok(())
    }

    pub fn tag(&mut self, term: &str, tag: &str) -> AppResultU {
        diesel_query!(tags [E R] {
            diesel::insert_into(d::tags)
                .values((d::term.eq(&term), d::tag.eq(&tag)))
                .execute(self.connection)?;
        });
        Ok(())
    }

    pub fn levelize(&mut self, level: u8, key: &str) -> AppResultU {
        diesel_query!(levels [E R] {
            diesel::replace_into(d::levels)
                .values((d::term.eq(&key), d::level.eq(i32::from(level))))
                .execute(self.connection)?;
        });
        Ok(())
    }
}



// TODO REMOVE ME
impl Default for Definition {
    fn default() -> Self {
        Definition { key: "dummy-key".to_owned(), content: vec![Text::Note("dummy-content".to_owned())] }
    }

}

impl Text {
    fn text_for_search(&self) -> Option<&str> {
        use self::Text::*;

        match self {
            Annot(s) | Definition(s) | Example(s) | Information(s) | Note(s) =>
                Some(s),
            Class(_) | Countability(_) | Error(_) | Etymology(_) | Tag(_) | Word(_) =>
                None,
        }
    }
}
