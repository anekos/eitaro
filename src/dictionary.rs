
use std::error::Error;
use std::path::Path;
use std::string::FromUtf8Error;

use rusty_leveldb::{DB, LdbIterator, Options, WriteBatch};



pub struct Dictionary {
    pub db: DB,
}

pub struct DictionaryWriter {
    pub write_batch: WriteBatch,
}



impl Dictionary {
    pub fn new<T: AsRef<Path>>(dictionary_path: &T) -> Result<Self, Box<Error>> {
        let opts = Options::default();
        // opts.compression_type = CompressionType::CompressionSnappy;
        // opts.reuse_logs = false;
        // opts.reuse_manifest = false;

        let result = Dictionary {
            db: DB::open(dictionary_path, opts)?,
        };
        Ok(result)
    }

    pub fn writes<F>(&mut self, mut f: F) -> Result<(), Box<Error>> where F: FnMut(&mut DictionaryWriter) {
        let mut writer = DictionaryWriter::new();
        f(&mut writer);
        self.db.write(writer.write_batch, false)?;
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Option<Result<String, FromUtf8Error>> {
        self.db.get(key.as_bytes()).map(String::from_utf8)
    }

    pub fn test(&mut self) -> Result<(), Box<Error>> {
        let mut iter = self.db.new_iter()?;
        while let Some((key, value)) = iter.next() {
            let key = String::from_utf8(key)?;
            let value = String::from_utf8(value)?;
            println!("{:?} → {:?}", key, value);
        }
        Ok(())
    }
}


impl DictionaryWriter {
    pub fn new() -> Self {
        DictionaryWriter {
            write_batch: WriteBatch::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        self.write_batch.put(key.as_bytes(), value.as_bytes());
    }
}
