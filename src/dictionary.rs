
use std::collections::{BTreeMap, HashSet};
use std::default::Default;
use std::path::{Path, PathBuf};

use array_tool::vec::Uniq;
use diesel::connection::Connection;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::run_pending_migrations;
use if_let_return::if_let_some;
use lazy_init::Lazy;
use regex::Regex;
use serde_derive::{Serialize, Deserialize};

use crate::correction::Corrector;
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

pub struct DictionaryWriter<'a> {
    connection: &'a SqliteConnection,
    alias_buffer: CatBuffer<String>,
    keys: HashSet<String>,
}

pub struct Stat {
    pub aliases: usize,
    pub words: usize,
}

#[derive(Default)]
struct CatBuffer<T> {
    buffer: BTreeMap<String, Vec<T>>,
}


impl Dictionary {
    pub fn new<T: AsRef<Path>>(dictionary_path: &T) -> Self {
        Dictionary {
            corrector: Lazy::new(),
            path: dictionary_path.as_ref().to_path_buf()
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

        let mut result = vec![];

        let connection = self.connect_db()?;

        if let Some(entry) = lookup_entry(&connection, word)? {
            result.push(entry);
        }

        let connection = self.connect_db()?;
        if let Some(aliases) = lookup_unaliased(&connection, word)? {
            for alias in aliases.split('\n') {
                if alias != word {
                    if let Some(entry) = lookup_entry(&connection, alias)? {
                        result.push(entry);
                    }
                }
            }
        }

        if result.is_empty() {
            for stemmed in stem(&word) {
                if let Some(entry) = lookup_entry(&connection, &stemmed)? {
                    result.push(entry);
                }
            }
        }

        Ok(opt(result))
   }

   pub fn get_level(&mut self, word: &str) -> AppResult<Option<u8>> {
       fn get_level(connection: &SqliteConnection, word: &str) -> AppResult<Option<u8>> {
           diesel_query!(levels, Level, {
               let found = levels
                   .filter(term.eq(word))
                   .limit(1)
                   .load::<Level>(connection)?;

               Ok(found.get(0).map(|it| it.level as u8))
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

    pub fn lemmatize(&mut self, word: &str) -> AppResult<String> {
        let connection = self.connect_db()?;
        lemmatize(&connection, word)
    }

    pub fn correct(&mut self, word: &str) -> Vec<String> {
        let corrector = self.corrector.get_or_create(|| {
            use crate::db::schema::definitions::dsl::definitions;
            use crate::db::model::Definition;
            use diesel::RunQueryDsl;

            let connection = self.connect_db()?;
            let defs = definitions.load::<Definition>(&connection)?;
            let keys = defs.into_iter().map(|it| it.term).collect();
            Ok(Corrector { keys })
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

    pub fn write<F>(&mut self, mut f: F) -> AppResult<Stat> where F: FnMut(&mut DictionaryWriter) -> AppResultU {
        let connection = self.connect_db()?;
        run_pending_migrations(&connection)?;

        connection.transaction::<_, AppError, _>(|| {
            use crate::db::schema;
            use diesel::RunQueryDsl;

            diesel::delete(schema::aliases::dsl::aliases).execute(&connection)?;
            diesel::delete(schema::definitions::dsl::definitions).execute(&connection)?;
            diesel::delete(schema::lemmatizations::dsl::lemmatizations).execute(&connection)?;
            diesel::delete(schema::levels::dsl::levels).execute(&connection)?;

            let mut writer = DictionaryWriter::new(&connection);
            f(&mut writer)?;
            writer.complete()
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
    let found = diesel_query!(definitions, Definition, {
        definitions
            .filter(term.eq(word))
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
    diesel_query!(lemmatizations, Lemmatization, {
        let found = lemmatizations
            .filter(source.eq(word))
            .limit(1)
            .load::<Lemmatization>(connection)?;

        Ok(found.get(0).map(|it| it.target.to_owned()))
    })
}

fn lookup_unaliased(connection: &SqliteConnection, word: &str) -> AppResult<Option<String>> {
    diesel_query!(aliases, Alias, {
        let found = aliases
            .filter(source.eq(word))
            .limit(1)
            .load::<Alias>(connection)?;

        Ok(found.get(0).map(|it| it.target.to_owned()))
    })
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
    fn new(connection: &'a SqliteConnection) -> Self {
        DictionaryWriter {
            alias_buffer: CatBuffer::default(),
            connection,
            keys: HashSet::default(),
        }
    }

    pub fn alias(&mut self, from: &str, to: &str, for_lemmatization: bool) -> AppResultU {
        if let (Some(from), Some(to)) = (fix_word(from), fix_word(to)) {
            if from == to {
                return Ok(());
            }

            if for_lemmatization {
                diesel_query!(lemmatizations, {
                    diesel::insert_into(lemmatizations).values((d::source.eq(&from), d::target.eq(&to))).execute(self.connection)?;
                })
            }

            self.alias_buffer.insert(from, to);
        }
        Ok(())
    }

    pub fn define(&mut self, key: &str, content: Vec<Text>) -> AppResultU {
        let lkey = key.to_lowercase();

        self.add_key(&lkey)?;

        let def = Definition { key: key.to_owned(), content };

        diesel_query!(definitions, {
            let serialized = serde_json::to_string(&def).unwrap();
            diesel::insert_into(definitions)
                .values((d::term.eq(lkey), d::definition.eq(serialized)))
                .execute(self.connection)?;
        });

        Ok(())
    }

    pub fn tag(&mut self, term: &str, tag: &str) -> AppResultU {
        diesel_query!(tags, {
            diesel::insert_into(tags).values((d::term.eq(&term), d::tag.eq(&tag))).execute(self.connection)?;
        });
        Ok(())
    }

    pub fn levelize(&mut self, level: u8, key: &str) -> AppResultU {
        diesel_query!(levels, {
            diesel::replace_into(levels).values((d::term.eq(&key), d::level.eq(i32::from(level)))).execute(self.connection)?;
        });
        Ok(())
    }

    fn add_key(&mut self, key: &str) -> AppResultU {
        if !(key.contains(' ') || key.contains('-') || key.contains('\'')) {
            self.keys.insert(key.to_string());
        }
        Ok(())
    }

    fn complete(self) -> AppResult<Stat> {
        let aliases = self.alias_buffer.complete(self.connection)?;
        Ok(Stat { aliases, words: 0 }) // FIXME
    }
}


impl<T> CatBuffer<T> {
    fn insert(&mut self, key: String, value: T) {
        let entries = self.buffer.entry(key).or_insert_with(|| vec![]);
        entries.push(value);
    }
}

impl CatBuffer<String> {
    fn complete(self, connection: &SqliteConnection) -> AppResult<usize> {
        diesel_query!(aliases, {
            let len = self.buffer.len();

            for (k, vs) in &self.buffer {
                for v in vs {
                    diesel::insert_into(aliases).values((d::source.eq(k), d::target.eq(v))).execute(connection)?;
                }
            }

            Ok(len)
        })
    }
}

// TODO REMOVE ME
impl Default for Definition {
    fn default() -> Self {
        Definition { key: "dummy-key".to_owned(), content: vec![Text::Note("dummy-content".to_owned())] }
    }

}
