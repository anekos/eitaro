
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::default::Default;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use array_tool::vec::Uniq;
use bincode::serialize;
use if_let_return::if_let_some;
use kv::bincode::Bincode;
use kv::{Bucket, Config, Error as KvError, Manager, Serde, Txn, ValueBuf};
use lazy_init::Lazy;
use regex::Regex;
use serde_derive::{Serialize, Deserialize};

use crate::correction::Corrector;
use crate::errors::{AppError, AppResult, AppResultU};
use crate::str_utils::{fix_word, shorten, uncase};



const ALIAS_BUCKET: &str = "alias";
const LEMMA_BUCKET: &str = "lemma";
const LEVEL_BUCKET: &str = "level";
const MAIN_BUCKET: &str = "dictionary";

type DicValue = ValueBuf<Bincode<Entry>>;
type LevelValue = ValueBuf<Bincode<u8>>;


pub struct Dictionary  {
    config: Config,
    corrector: Lazy<AppResult<Corrector>>,
    manager: Manager,
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
    alias_bucket: Bucket<'a, String, String>,
    alias_buffer: CatBuffer<String>,
    keys: HashSet<String>,
    lemma_bucket: Bucket<'a, String, String>,
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
        config.bucket(LEMMA_BUCKET, None);
        config.bucket(LEVEL_BUCKET, None);

        Dictionary {
            config,
            corrector: Lazy::new(),
            manager,
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

        let handle = self.manager.open(self.config.clone())?;
        let store = handle.read()?;
        let main_bucket = store.bucket::<String, DicValue>(Some(MAIN_BUCKET))?;
        let alias_bucket = store.bucket::<String, String>(Some(ALIAS_BUCKET))?;
        let transaction = store.read_txn()?;

        let mut result = vec![];

        if let Some(entry) = fix(transaction.get(&main_bucket, word.to_owned()))? {
            result.push(entry);
        }

        if let Some(aliases) = fix_alias(transaction.get(&alias_bucket, word.to_owned()))? {
            for alias in aliases.split('\n') {
                if alias != word {
                    if let Some(entry) = fix(transaction.get(&main_bucket, alias.to_owned()))? {
                        result.push(entry);
                    }
                }
            }
        }

        if result.is_empty() {
            let stemmed = stem(&word).to_string();
            if let Some(entry) = fix(transaction.get(&main_bucket, stemmed))? {
                result.push(entry);
            }
        }

        Ok(opt(result))
   }

   pub fn get_level(&mut self, word: &str) -> AppResult<Option<u8>> {
       fn get_level(tx: &Txn<'_>, bkt: &Bucket<'_, String, LevelValue>, word: &str) -> AppResult<Option<u8>> {
           match tx.get(&bkt, word.to_owned()) {
               Ok(found) => Ok(Some(found.inner()?.to_serde())),
               Err(KvError::NotFound) => Ok(None),
               Err(err) => Err(AppError::from(err)),
           }
       }

       let handle = self.manager.open(self.config.clone())?;
       let store = handle.read()?;
       let level_bucket = store.bucket::<String, LevelValue>(Some(LEVEL_BUCKET))?;
       let lemma_bucket = store.bucket::<String, String>(Some(LEMMA_BUCKET))?;
       let transaction = store.read_txn()?;

       let found = get_level(&transaction, &level_bucket, word)?;
       if found.is_some() {
           return Ok(found)
       }

       if let Some(lemmed) = lemmatize(&transaction, &lemma_bucket, word)? {
           return get_level(&transaction, &level_bucket, &lemmed);
       }

       Ok(None)
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

    pub fn lemmatize(&mut self, word: &str) -> AppResult<Option<String>> {
        let handle = self.manager.open(self.config.clone())?;
        let store = handle.read()?;
        let lemma_bucket = store.bucket::<String, String>(Some(LEMMA_BUCKET))?;
        let transaction = store.read_txn()?;
        lemmatize(&transaction, &lemma_bucket, word)
    }

    pub fn correct(&mut self, word: &str) -> Vec<String> {
        let mut path = self.path.clone();
        path.push("keys");

        let corrector = self.corrector.get_or_create(|| {
            Corrector::load(&path)
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
        let handle = self.manager.open(self.config.clone())?;
        let store = handle.write()?;
        let main_bucket = store.bucket::<String, DicValue>(Some(MAIN_BUCKET))?;
        let alias_bucket = store.bucket::<String, String>(Some(ALIAS_BUCKET))?;
        let level_bucket = store.bucket::<String, LevelValue>(Some(LEVEL_BUCKET))?;
        let lemma_bucket = store.bucket::<String, String>(Some(LEMMA_BUCKET))?;

        let mut transaction = store.write_txn()?;
        transaction.clear_db(&alias_bucket)?;
        transaction.clear_db(&lemma_bucket)?;
        transaction.clear_db(&level_bucket)?;
        transaction.clear_db(&main_bucket)?;
        transaction.commit()?;

        let transaction = store.write_txn()?;
        let mut writer = DictionaryWriter::new(transaction, main_bucket, alias_bucket, lemma_bucket, level_bucket, &self.path);
        f(&mut writer)?;
        writer.complete()
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


fn fix(result: Result<DicValue, KvError>) -> AppResult<Option<Entry>> {
    match result {
        Ok(found) => Ok(Some(found.inner()?.to_serde())),
        Err(KvError::NotFound) => Ok(None),
        Err(err) => Err(AppError::from(err)),
    }
}

fn fix_alias(result: Result<String, KvError>) -> AppResult<Option<String>> {
    match result {
        Ok(found) => Ok(Some(found)),
        Err(KvError::NotFound) => Ok(None),
        Err(err) => Err(AppError::from(err)),
    }
}

fn lemmatize<'a>(tx: &Txn<'a>, bkt: &Bucket<'a, String, String>, word: &str) -> AppResult<Option<String>> {
    let mut word = word.to_owned();
    let mut path = HashSet::<String>::new();

    path.insert(word.clone());

    while let Some(found) = fix_alias(tx.get(&bkt, word.clone()))? {
        if !path.insert(found.clone()) {
            return Ok(Some(word))
        }
        word = found;
    }
    Ok(Some(stem(&word).to_string()))
}

fn stem(word: &str) -> Cow<'_, str> {
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

    let wlen = word.len();

    for (suffix, to) in &pairs {
        if wlen < suffix.len() + 2 {
            break;
        }

        if word.ends_with(suffix) {
            return format!(
                "{}{}",
                &word[0 .. wlen - suffix.len()],
                to).into();
        }
    }

    word.into()
}




impl<'a> DictionaryWriter<'a> {
    fn new<T: AsRef<Path>>(transaction: Txn<'a>, main_bucket: Bucket<'a, String, DicValue>, alias_bucket: Bucket<'a, String, String>, lemma_bucket: Bucket<'a, String, String>, level_bucket: Bucket<'a, String, LevelValue>, path: &'a T) -> Self {
        DictionaryWriter {
            alias_bucket,
            alias_buffer: CatBuffer::default(),
            keys: HashSet::default(),
            lemma_bucket,
            level_bucket,
            level_buffer: HashMap::default(),
            main_bucket,
            main_buffer: CatBuffer::default(),
            transaction,
            path,
        }
    }

    pub fn insert(&mut self, key: &str, content: Vec<Text>) -> AppResultU {
        let lkey = key.to_lowercase();
        if !(key.contains(' ') || key.contains('-') || key.contains('\'')) {
            self.keys.insert(lkey.clone());
        }
        self.main_buffer.insert(lkey, Definition { key: key.to_owned(), content });
        Ok(())
    }

    pub fn alias(&mut self, from: &str, to: &str, for_lemmatization: bool) -> AppResultU {
        if let (Some(from), Some(to)) = (fix_word(from), fix_word(to)) {
            if from == to {
                return Ok(());
            }

            if for_lemmatization {
                self.transaction.set(&self.lemma_bucket, from.clone(), to.clone())?;
            }
            self.alias_buffer.insert(from, to);
        }
        Ok(())
    }

    pub fn levelize(&mut self, level: u8, key: &str) -> AppResultU {
        self.transaction.set(&self.level_bucket, key.to_owned(), Bincode::to_value_buf(level)?)?;
        let lb = self.level_buffer.entry(level).or_default();
        lb.push(key.to_owned());
        Ok(())
    }

    fn complete(mut self) -> AppResult<Stat> {
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

        {
            let mut path = self.path.as_ref().to_path_buf();
            path.push("keys");
            let file = OpenOptions::new().write(true).append(false).create(true).truncate(true).open(path)?;
            let mut file = BufWriter::new(file);
            let buffer: Vec<u8> = serialize(&self.keys)?;
            file.write_all(&buffer)?;
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
    fn complete<'a>(self, transaction: &mut Txn<'a>, bucket: &Bucket<'a, String, DicValue>) -> AppResult<usize> {
        let len = self.buffer.len();
        for (key, definitions) in self.buffer {
            transaction.set(bucket, key.clone(), Bincode::to_value_buf(Entry { key,  definitions })?)?;
        }
        Ok(len)
    }
}

impl CatBuffer<String> {
    fn complete<'a>(self, transaction: &mut Txn<'a>, bucket: &Bucket<'a, String, String>) -> AppResult<usize> {
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
