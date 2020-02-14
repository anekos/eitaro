
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread::spawn;

use structopt::StructOpt;

mod curses;
pub mod color;
pub mod gui;
pub mod plain;

use crate::dictionary::Entry;



#[derive(StructOpt, Debug)]
pub enum Opt {
    /// Color
    Color,
    /// Curses
    Curses(curses::Opt),
    /// GUI
    Gui(gui::Opt),
    /// Plain
    Plain,
}

#[derive(Clone)]
pub struct Screen {
    tx: SyncSender<Option<Vec<Entry>>>,
}

impl Screen {
    pub fn new(opt: Opt, dictionary_path: PathBuf, bind_to: String) -> Self {
        use self::Opt::*;

        let (tx, rx) = sync_channel(0);

        let screen = Screen { tx: tx.clone() };

        spawn(move || match opt {
            Curses(opt) =>
                curses::main(&rx, opt, &bind_to),
            Color =>
                color::main(rx).unwrap(),
            Gui(opt) =>
                gui::main(tx, rx, opt, dictionary_path),
            Plain =>
                plain::main(rx).unwrap(),
        });

        screen
    }

    pub fn print_opt(&self, content: Option<Vec<Entry>>) {
        self.tx.send(content).unwrap();
    }
}
