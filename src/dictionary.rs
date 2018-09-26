
use std::collections::BTreeMap;
use std::path::Path;

use kv::{Bucket, Config, Manager, Txn, Error as KvError};
use regex::Regex;

use errors::AppError;



const MAIN_BUCKET: &str = "dictionary";
const ALIAS_BUCKET: &str = "alias";


pub struct Dictionary {
    manager: Manager,
    config: Config,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub key: String,
    pub content: String,
}

pub struct DictionaryWriter<'a> {
    alias_bucket: Bucket<'a, String, String>,
    main_bucket: Bucket<'a, String, String>,
    merge_table: BTreeMap<String, Vec<String>>,
    transaction: Txn<'a>,
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

        if result.is_some() {
            return Ok(result)
        }


        let splitter = Regex::new(r"[-#'=\s]+")?;
        let mut candidates: Vec<&str> = splitter.split(word).collect();
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
        let transaction = store.write_txn()?;

        let mut writer = DictionaryWriter::new(transaction, main_bucket, alias_bucket);
        f(&mut writer)?;
        writer.complete()?;

        Ok(())
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

        let word = word.to_lowercase();
        let handle = self.manager.open(self.config.clone())?;
        let store = handle.read()?;
        let main_bucket = store.bucket::<String, String>(Some(MAIN_BUCKET))?;
        let alias_bucket = store.bucket::<String, String>(Some(ALIAS_BUCKET))?;
        let transaction = store.read_txn()?;

        let mut result = vec![];

        if let Some(content) = fix(transaction.get(&main_bucket, word.clone()))? {
            result.push(Entry { key: word.clone(), content });
        }

        if_let_some!(alias = fix(transaction.get(&alias_bucket, word.clone()))?, Ok(opt(result)));
        if alias == word {
            return Ok(opt(result));
        }
        if_let_some!(content = fix(transaction.get(&main_bucket, alias.clone()))?, Ok(opt(result)));
        result.push(Entry { key: alias, content });

        Ok(opt(result))
    }
}

impl<'a> DictionaryWriter<'a> {
    fn new(transaction: Txn<'a>, main_bucket: Bucket<'a, String, String>, alias_bucket: Bucket<'a, String, String>) -> Self {
        DictionaryWriter {
            alias_bucket,
            main_bucket,
            merge_table: BTreeMap::default(),
            transaction,
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) -> Result<(), AppError> {
        let key = key.to_lowercase();

        let entries = self.merge_table.entry(key).or_insert_with(|| vec![]);
        entries.push(value.to_owned());
        Ok(())
    }

    pub fn alias(&mut self, from: &str, to: &str) -> Result<(), AppError> {
        self.transaction.set(&self.alias_bucket, from.to_lowercase(), to.to_lowercase())?;
        Ok(())
    }

    fn complete(mut self) -> Result<(), AppError> {
        for (key, mut values) in self.merge_table {
            values.push("".to_string());
            self.transaction.set(&self.main_bucket, key, values.join("\n"))?;
        }

        self.transaction.commit()?;
        Ok(())
    }
}
