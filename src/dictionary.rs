
use std::error::Error;
use std::mem::swap;
use std::path::Path;
use std::string::FromUtf8Error;

use rusty_leveldb::{DB, Options, WriteBatch};



pub struct Dictionary {
    pub db: DB,
}

#[derive(Default)]
pub struct MergeBuffer {
    buffered: Option<String>,
    entries: Vec<String>,
}

pub struct DictionaryWriter<'a> {
    db: &'a mut DB,
    batch_size: usize,
    merge_buffer: MergeBuffer,
    write_batch: WriteBatch,
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
        let mut writer = DictionaryWriter::new(&mut self.db);
        f(&mut writer);
        writer.complete()?;
        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Option<Result<String, FromUtf8Error>> {
        self.db.get(key.as_bytes()).map(String::from_utf8)
    }

    // pub fn test(&mut self) -> Result<(), Box<Error>> {
    //     let mut iter = self.db.new_iter()?;
    //     while let Some((key, value)) = iter.next() {
    //         let key = String::from_utf8(key)?;
    //         let value = String::from_utf8(value)?;
    //         println!("{:?} → {:?}", key, value);
    //     }
    //     Ok(())
    // }
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


impl<'a> DictionaryWriter<'a> {
    fn new(db: &'a mut DB) -> Self {
        DictionaryWriter {
            batch_size: 0,
            db,
            merge_buffer: MergeBuffer::default(),
            write_batch: WriteBatch::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: &str) {
        if let Some((key, values)) = self.merge_buffer.insert(key, value) {
            self.write_batch.put(key.as_bytes(), values.join("\n").as_bytes());
            if 10000 <= self.batch_size {
                self.flush();
            } else {
                self.batch_size += 1;
            }
        }
    }

    fn complete(&mut self) -> Result<(), Box<Error>> {
        if let Some((key, values)) = self.merge_buffer.flush() {
            self.write_batch.put(key.as_bytes(), values.join("\n").as_bytes());
        }
        self.flush();
        self.db.flush()?;
        Ok(())
    }

    fn flush(&mut self) {
        let mut batch = WriteBatch::new();
        swap(&mut batch, &mut self.write_batch);
        self.db.write(batch, false).unwrap();
        self.batch_size = 0;
    }
}
