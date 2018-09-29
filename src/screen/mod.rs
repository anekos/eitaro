
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread::spawn;

mod curses;
mod parser;
pub mod standard;

use dictionary::Entry;



pub struct Screen {
    tx: SyncSender<Option<Vec<Entry>>>,
}

impl Screen {
    pub fn new(curses: bool, kuru: bool, bind_to: String) -> Self {
        let (tx, rx) = sync_channel(0);

        if curses {
            spawn(move || curses::main(rx, kuru, &bind_to));
        } else {
            spawn(|| standard::main(rx));
        }

        Screen { tx }
    }

    pub fn print_opt(&self, content: Option<Vec<Entry>>) {
        self.tx.send(content).unwrap();
    }
}
