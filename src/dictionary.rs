
use std::error::Error;

use rusty_leveldb::{DB, Options, WriteBatch, CompressionType};



pub struct Dictionary {
    pub db: DB,
}

pub struct DictionaryWriter {
    pub write_batch: WriteBatch,
}



impl Dictionary {
    pub fn new() -> Result<Self, Box<Error>> {
        let mut opts = Options::default();
        opts.compression_type = CompressionType::CompressionSnappy;

        let result = Dictionary {
            db: DB::open("eitaro.dic", opts)?,
        };

        Ok(result)
    }

    pub fn writes<F>(&mut self, mut f: F) -> Result<(), Box<Error>> where F: FnMut(&mut DictionaryWriter) {
        let mut writer = DictionaryWriter::new();
        f(&mut writer);
        self.db.write(writer.write_batch, false)?;
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
