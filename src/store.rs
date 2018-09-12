
use std::mem::swap;
use std::path::Path;

use kv::{Bucket, Config, Error, Manager, Txn};



const BUCKET_NAME: &str = "dictionary";


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
    bucket: Bucket<'a, String, String>,
    transaction: Txn<'a>,
    merge_buffer: MergeBuffer,
}


impl Dictionary {
    pub fn new<T: AsRef<Path>>(dictionary_path: &T) -> Self {
        let manager = Manager::new();
        let mut config = Config::default(dictionary_path);
        config.bucket(BUCKET_NAME, None);

        Dictionary { manager, config }
    }

    pub fn writes<F>(&mut self, mut f: F) -> Result<(), Error> where F: FnMut(&mut DictionaryWriter) {
        let handle = self.manager.open(self.config.clone())?;
        let store = handle.write()?;
        let bucket = store.bucket::<String, String>(Some(BUCKET_NAME))?;
        let transaction = store.write_txn()?;

        let mut writer = DictionaryWriter::new(bucket, transaction);
        f(&mut writer);
        writer.complete()?;

        Ok(())
    }

    pub fn get(&mut self, word: String) -> Result<String, Error> {
        let handle = self.manager.open(self.config.clone())?;
        let store = handle.read()?;
        let bucket = store.bucket::<String, String>(Some(BUCKET_NAME))?;
        let transaction = store.read_txn()?;

        transaction.get(&bucket, word)
    }
}

impl<'a> DictionaryWriter<'a> {
    fn new(bucket: Bucket<'a, String, String>, transaction: Txn<'a>) -> Self {
        DictionaryWriter {
            bucket,
            transaction,
            merge_buffer: MergeBuffer::default(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) -> Result<(), Error> {
        if let Some((key, values)) = self.merge_buffer.insert(key, value) {
            self.transaction.set(&self.bucket, key.to_owned(), values.join("\n"))?;
        }
        Ok(())
    }

    fn complete(mut self) -> Result<(), Error> {
        if let Some((key, values)) = self.merge_buffer.flush() {
            self.transaction.set(&self.bucket, key, values.join("\n"))?;
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
