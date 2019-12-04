
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread::spawn;

mod curses;
pub mod color;
pub mod gui;
pub mod plain;

use crate::dictionary::Entry;



pub enum ScreenConfig {
    Color,
    Curses { kuru: bool },
    Gui { font_name: Option<String>, font_size: f64 },
    Plain,
}



#[derive(Clone)]
pub struct Screen {
    tx: SyncSender<Option<Vec<Entry>>>,
}

impl Screen {
    pub fn new(config: ScreenConfig, bind_to: String) -> Self {
        use self::ScreenConfig::*;

        let (tx, rx) = sync_channel(0);

        spawn(move || match config {
            Curses { kuru } =>
                curses::main(&rx, kuru, &bind_to),
            Color =>
                color::main(rx).unwrap(),
            Gui{ font_name, font_size } =>
                gui::main(rx, font_name, font_size),
            Plain =>
                plain::main(rx).unwrap(),
        });

        Screen { tx }
    }

    pub fn print_opt(&self, content: Option<Vec<Entry>>) {
        self.tx.send(content).unwrap();
    }
}
