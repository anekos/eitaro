
use std::collections::BTreeMap;
use std::path::Path;

use array_tool::vec::Uniq;
use kv::{Bucket, Config, Manager, Txn, Error as KvError};
use regex::Regex;

use errors::AppError;
use str_utils::{fix_word, shortened};



const MAIN_BUCKET: &str = "dictionary";
const ALIAS_BUCKET: &str = "alias";


pub struct Dictionary {
    manager: Manager,
    config: Config,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub key: String,
    pub content: String,
}

pub struct DictionaryWriter<'a> {
    alias_bucket: Bucket<'a, String, String>,
    alias_buffer: CatBuffer,
    main_bucket: Bucket<'a, String, String>,
    main_buffer: CatBuffer,
    transaction: Txn<'a>,
}

#[derive(Default)]
struct CatBuffer {
    buffer: BTreeMap<String, Vec<String>>,
}


impl Dictionary {
    pub fn new<T: AsRef<Path>>(dictionary_path: &T) -> Self {
        let manager = Manager::new();
        let mut config = Config::default(dictionary_path);
        config.bucket(MAIN_BUCKET, None);
        config.bucket(ALIAS_BUCKET, None);

        Dictionary { manager, config }
    }

    pub fn get_smart(&mut self, word: &str) -> Result<Option<Vec<Entry>>, AppError> {
        if_let_some!(word = fix_word(word), Ok(None));

        for word in shortened(&word) {
            let mut result = self.get_similars(&word)?;
            if let Some(result) = result.as_mut() {
                return Ok(Some(result.unique()))
            }
        }

        let splitter = Regex::new(r"[-#'=\s]+")?;
        let mut candidates: Vec<&str> = splitter.split(&word).collect();
        candidates.sort_by(|a, b| a.len().cmp(&b.len()).reverse());
        for candidate in candidates {
            let result = self.get(candidate)?;
            if result.is_some() {
                return Ok(result);
            }
        }

        Ok(None)
    }

    pub fn writes<F>(&mut self, mut f: F) -> Result<(), AppError> where F: FnMut(&mut DictionaryWriter) -> Result<(), AppError> {
        let handle = self.manager.open(self.config.clone())?;
        let store = handle.write()?;
        let main_bucket = store.bucket::<String, String>(Some(MAIN_BUCKET))?;
        let alias_bucket = store.bucket::<String, String>(Some(ALIAS_BUCKET))?;
        let mut transaction = store.write_txn()?;

        transaction.clear_db(&main_bucket)?;
        transaction.clear_db(&alias_bucket)?;

        let mut writer = DictionaryWriter::new(transaction, main_bucket, alias_bucket);
        f(&mut writer)?;
        writer.complete()?;

        Ok(())
    }

    fn get_similars(&mut self, word: &str) -> Result<Option<Vec<Entry>>, AppError> {
        let mut result = self.get(word)?;

        {
            let mut mutated = vec![];
            let chars = ['-', ',', '\'', '_', '=', ' '];
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

    fn get(&mut self, word: &str) -> Result<Option<Vec<Entry>>, AppError> {
        fn fix(result: Result<String, KvError>) -> Result<Option<String>, KvError> {
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
        let main_bucket = store.bucket::<String, String>(Some(MAIN_BUCKET))?;
        let alias_bucket = store.bucket::<String, String>(Some(ALIAS_BUCKET))?;
        let transaction = store.read_txn()?;

        let mut result = vec![];

        if let Some(content) = fix(transaction.get(&main_bucket, word.to_owned()))? {
            result.push(Entry { key: word.to_owned(), content });
        }

        if_let_some!(aliases = fix(transaction.get(&alias_bucket, word.to_owned()))?, Ok(opt(result)));
        for alias in aliases.split('\n') {
            if alias != word {
                if_let_some!(content = fix(transaction.get(&main_bucket, alias.to_owned()))?, Ok(opt(result)));
                result.push(Entry { key: alias.to_owned(), content });
            }
        }

        Ok(opt(result))
    }
}


impl<'a> DictionaryWriter<'a> {
    fn new(transaction: Txn<'a>, main_bucket: Bucket<'a, String, String>, alias_bucket: Bucket<'a, String, String>) -> Self {
        DictionaryWriter {
            alias_bucket,
            alias_buffer: CatBuffer::default(),
            main_bucket,
            main_buffer: CatBuffer::default(),
            transaction,
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) -> Result<(), AppError> {
        let key = key.to_lowercase();
        self.main_buffer.insert(key, value.to_owned());
        Ok(())
    }

    pub fn alias(&mut self, from: &str, to: &str) -> Result<(), AppError> {
        if let (Some(from), Some(to)) = (fix_word(from), fix_word(to)) {
            self.alias_buffer.insert(from, to);
        }
        Ok(())
    }

    fn complete(mut self) -> Result<(), AppError> {
        self.main_buffer.complete(&mut self.transaction, &self.main_bucket, true)?;
        self.alias_buffer.complete(&mut self.transaction, &self.alias_bucket, false)?;
        self.transaction.commit()?;
        Ok(())
    }
}


impl CatBuffer {
    fn insert(&mut self, key: String, value: String) {
        let entries = self.buffer.entry(key).or_insert_with(|| vec![]);
        entries.push(value.to_owned());
    }

    fn complete<'a>(self, transaction: &mut Txn<'a>, bucket: &Bucket<'a, String, String>, last_line_break: bool) -> Result<(), AppError> {
        for (key, mut values) in self.buffer {
            if last_line_break {
                values.push("".to_string());
            }
            transaction.set(bucket, key, values.join("\n"))?;
        }
        Ok(())
    }
}
