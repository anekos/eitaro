
use std::collections::{BTreeMap, HashMap};
use std::default::Default;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use array_tool::vec::Uniq;
use if_let_return::if_let_some;
use kv::bincode::Bincode;
use kv::{Bucket, Config, Error as KvError, Manager, Serde, Txn, ValueBuf};
use regex::Regex;
use serde_derive::{Serialize, Deserialize};

use crate::errors::AppError;
use crate::str_utils::{fix_word, shorten, uncase};



const MAIN_BUCKET: &str = "dictionary";
const ALIAS_BUCKET: &str = "alias";
const LEVEL_BUCKET: &str = "level";

type DicValue = ValueBuf<Bincode<Entry>>;
type LevelValue = ValueBuf<Bincode<u8>>;


pub struct Dictionary {
    config: Config,
    manager: Manager,
    path: PathBuf,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Text {
    Annot(String),
    Countability(char),
    Class(String),
    Definition(String),
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
    alias_bucket: Bucket<'a, String, String>,
    alias_buffer: CatBuffer<String>,
    level_bucket: Bucket<'a, String, LevelValue>,
    level_buffer: HashMap<u8, Vec<String>>,
    main_bucket: Bucket<'a, String, DicValue>,
    main_buffer: CatBuffer<Definition>,
    transaction: Txn<'a>,
    path: &'a dyn AsRef<Path>,
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
        let manager = Manager::new();
        let mut config = Config::default(dictionary_path);
        config.bucket(MAIN_BUCKET, None);
        config.bucket(ALIAS_BUCKET, None);
        config.bucket(LEVEL_BUCKET, None);

        Dictionary {
            manager,
            config,
            path: dictionary_path.as_ref().to_path_buf()
        }
    }

   pub fn get(&mut self, word: &str) -> Result<Option<Vec<Entry>>, AppError> {
        fn fix(result: Result<DicValue, KvError>) -> Result<Option<Entry>, KvError> {
            match result {
                Ok(found) => Ok(Some(found.inner()?.to_serde())),
                Err(KvError::NotFound) => Ok(None),
                Err(err) => Err(err),
            }
        }

        fn fix_alias(result: Result<String, KvError>) -> Result<Option<String>, KvError> {
            match result {
                Ok(found) => Ok(Some(found)),
                Err(KvError::NotFound) => Ok(None),
                Err(err) => Err(err),
            }
        }

        fn opt(result: Vec<Entry>) -> Option<Vec<Entry>> {
            if result.is_empty() {
                return None;
            }
            Some(result)
        }

        let handle = self.manager.open(self.config.clone())?;
        let store = handle.read()?;
        let main_bucket = store.bucket::<String, DicValue>(Some(MAIN_BUCKET))?;
        let alias_bucket = store.bucket::<String, String>(Some(ALIAS_BUCKET))?;
        let transaction = store.read_txn()?;

        let mut result = vec![];

        if let Some(entry) = fix(transaction.get(&main_bucket, word.to_owned()))? {
            result.push(entry);
        }

        if_let_some!(aliases = fix_alias(transaction.get(&alias_bucket, word.to_owned()))?, Ok(opt(result)));
        for alias in aliases.split('\n') {
            if alias != word {
                if_let_some!(entry = fix(transaction.get(&main_bucket, alias.to_owned()))?, Ok(opt(result)));
                result.push(entry);
            }
        }

        Ok(opt(result))
    } pub fn get_smart(&mut self, word: &str) -> Result<Option<Vec<Entry>>, AppError> {
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

    pub fn write<F>(&mut self, mut f: F) -> Result<Stat, AppError> where F: FnMut(&mut DictionaryWriter) -> Result<(), AppError> {
        let handle = self.manager.open(self.config.clone())?;
        let store = handle.write()?;
        let main_bucket = store.bucket::<String, DicValue>(Some(MAIN_BUCKET))?;
        let alias_bucket = store.bucket::<String, String>(Some(ALIAS_BUCKET))?;
        let level_bucket = store.bucket::<String, LevelValue>(Some(LEVEL_BUCKET))?;

        let mut transaction = store.write_txn()?;
        transaction.clear_db(&main_bucket)?;
        transaction.clear_db(&alias_bucket)?;
        transaction.commit()?;

        let transaction = store.write_txn()?;
        let mut writer = DictionaryWriter::new(transaction, main_bucket, alias_bucket, level_bucket, &self.path);
        f(&mut writer)?;
        writer.complete()
    }

    fn get_similars(&mut self, word: &str) -> Result<Option<Vec<Entry>>, AppError> {
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


impl<'a> DictionaryWriter<'a> {
    fn new<T: AsRef<Path>>(transaction: Txn<'a>, main_bucket: Bucket<'a, String, DicValue>, alias_bucket: Bucket<'a, String, String>, level_bucket: Bucket<'a, String, LevelValue>, path: &'a T) -> Self {
        DictionaryWriter {
            alias_bucket,
            alias_buffer: CatBuffer::default(),
            level_bucket,
            level_buffer: HashMap::default(),
            main_bucket,
            main_buffer: CatBuffer::default(),
            transaction,
            path,
        }
    }

    pub fn insert(&mut self, key: &str, content: Vec<Text>) -> Result<(), AppError> {
        self.main_buffer.insert(key.to_lowercase(), Definition { key: key.to_owned(), content });
        Ok(())
    }

    pub fn alias(&mut self, from: &str, to: &str) -> Result<(), AppError> {
        if let (Some(from), Some(to)) = (fix_word(from), fix_word(to)) {
            self.alias_buffer.insert(from, to.to_owned());
        }
        Ok(())
    }

    pub fn levelize(&mut self, level: u8, key: &str) -> Result<(), AppError> {
        self.transaction.set(&self.level_bucket, key.to_owned(), Bincode::to_value_buf(level)?)?;
        let lb = self.level_buffer.entry(level).or_default();
        lb.push(key.to_owned());
        Ok(())
    }

    fn complete(mut self) -> Result<Stat, AppError> {
        let words = self.main_buffer.complete(&mut self.transaction, &self.main_bucket)?;
        let aliases = self.alias_buffer.complete(&mut self.transaction, &self.alias_bucket)?;
        self.transaction.commit()?;


        for (level, words) in self.level_buffer {
            let mut path = self.path.as_ref().to_path_buf();
            path.push(format!("level-{}", level));
            let file = OpenOptions::new().write(true).append(false).create(true).open(path)?;
            let mut file = BufWriter::new(file);
            for word in words {
                writeln!(file, "{}", word)?;
            }
        }

        Ok(Stat { aliases, words })
    }
}


impl<T> CatBuffer<T> {
    fn insert(&mut self, key: String, value: T) {
        let entries = self.buffer.entry(key).or_insert_with(|| vec![]);
        entries.push(value);
    }
}

impl CatBuffer<Definition> {
    fn complete<'a>(self, transaction: &mut Txn<'a>, bucket: &Bucket<'a, String, DicValue>) -> Result<usize, AppError> {
        let len = self.buffer.len();
        for (key, definitions) in self.buffer {
            transaction.set(bucket, key.clone(), Bincode::to_value_buf(Entry { key,  definitions })?)?;
        }
        Ok(len)
    }
}

impl CatBuffer<String> {
    fn complete<'a>(self, transaction: &mut Txn<'a>, bucket: &Bucket<'a, String, String>) -> Result<usize, AppError> {
        let len = self.buffer.len();
        for (key, values) in self.buffer {
            transaction.set(bucket, key, values.join("\n"))?;
        }
        Ok(len)
    }
}

// TODO REMOVE ME
impl Default for Definition {
    fn default() -> Self {
        Definition { key: "dummy-key".to_owned(), content: vec![Text::Note("dummy-content".to_owned())] }
    }

}
