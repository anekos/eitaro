
use std::mem::swap;
use std::path::Path;

use kv::{Bucket, Config, Manager, Txn, Error as KvError};

use errors::AppError;



const MAIN_BUCKET: &str = "dictionary";
const ALIAS_BUCKET: &str = "alias";


pub struct Dictionary {
    manager: Manager,
    config: Config,
}

#[derive(Default)]
pub struct MergeBuffer {
    buffered: Option<String>,
    entries: Vec<String>,
}

pub struct DictionaryWriter<'a> {
    transaction: Txn<'a>,
    main_bucket: Bucket<'a, String, String>,
    alias_bucket: Bucket<'a, String, String>,
    merge_buffer: MergeBuffer,
}


impl Dictionary {
    pub fn new<T: AsRef<Path>>(dictionary_path: &T) -> Self {
        let manager = Manager::new();
        let mut config = Config::default(dictionary_path);
        config.bucket(MAIN_BUCKET, None);
        config.bucket(ALIAS_BUCKET, None);

        Dictionary { manager, config }
    }

    pub fn writes<F>(&mut self, mut f: F) -> Result<(), AppError> where F: FnMut(&mut DictionaryWriter) -> Result<(), AppError> {
        let handle = self.manager.open(self.config.clone())?;
        let store = handle.write()?;
        let main_bucket = store.bucket::<String, String>(Some(MAIN_BUCKET))?;
        let alias_bucket = store.bucket::<String, String>(Some(ALIAS_BUCKET))?;
        let transaction = store.write_txn()?;

        let mut writer = DictionaryWriter::new(transaction, main_bucket, alias_bucket);
        f(&mut writer)?;
        writer.complete()?;

        Ok(())
    }

    pub fn get(&mut self, word: &str) -> Result<Option<String>, AppError> {
        fn fix(result: Result<String, KvError>) -> Result<Option<String>, KvError> {
            match result {
                Ok(found) => Ok(Some(found)),
                Err(KvError::NotFound) => Ok(None),
                Err(err) => Err(err),
            }
        }

        fn opt(result: &[String]) -> Option<String> {
            if result.is_empty() {
                return None;
            }
            Some(result.join("\n"))
        }

        let word = word.to_lowercase();
        let handle = self.manager.open(self.config.clone())?;
        let store = handle.read()?;
        let main_bucket = store.bucket::<String, String>(Some(MAIN_BUCKET))?;
        let alias_bucket = store.bucket::<String, String>(Some(ALIAS_BUCKET))?;
        let transaction = store.read_txn()?;

        let mut result = vec![];

        if let Some(ref main) = fix(transaction.get(&main_bucket, word.clone()))? {
            result.push(format!("#{}\n{}\n", word, main));
        }

        if_let_some!(alias = fix(transaction.get(&alias_bucket, word.clone()))?, Ok(opt(&result)));
        if alias == word {
            return Ok(opt(&result));
        }
        if_let_some!(aliased = fix(transaction.get(&main_bucket, alias.clone()))?, Ok(opt(&result)));
        result.push(format!("#{}\n{}", alias, aliased));

        Ok(opt(&result))
    }
}

impl<'a> DictionaryWriter<'a> {
    fn new(transaction: Txn<'a>, main_bucket: Bucket<'a, String, String>, alias_bucket: Bucket<'a, String, String>) -> Self {
        DictionaryWriter {
            transaction,
            main_bucket,
            alias_bucket,
            merge_buffer: MergeBuffer::default(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) -> Result<(), AppError> {
        let key = key.to_lowercase();

        if let Some((key, values)) = self.merge_buffer.insert(&key, value) {
            self.transaction.set(&self.main_bucket, key.to_owned(), values.join("\n"))?;
        }
        Ok(())
    }

    pub fn alias(&mut self, from: &str, to: &str) -> Result<(), AppError> {
        self.transaction.set(&self.alias_bucket, from.to_lowercase(), to.to_lowercase())?;
        Ok(())
    }

    fn complete(mut self) -> Result<(), AppError> {
        if let Some((key, values)) = self.merge_buffer.flush() {
            self.transaction.set(&self.main_bucket, key, values.join("\n"))?;
        }
        self.transaction.commit()?;
        Ok(())
    }
}

impl MergeBuffer {
    fn insert(&mut self, key: &str, value: &str) -> Option<(String, Vec<String>)> {
        if let Some(buffered) = self.buffered.as_ref() {
            if buffered == key {
                self.entries.push(value.to_owned());
                return None;
            }
        }

        let result = self.flush();
        self.buffered = Some(key.to_owned());
        self.entries.push(value.to_owned());
        result
    }

    fn flush(&mut self) -> Option<(String, Vec<String>)> {
        if let Some(buffered) = self.buffered.take() {
            let mut result = vec![];
            let old_key = buffered.clone();
            swap(&mut self.entries, &mut result);
            Some((old_key, result))
        } else {
            None
        }
    }
}
