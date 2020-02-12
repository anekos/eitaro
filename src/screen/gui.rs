
use std::fmt::Write;
use std::path::PathBuf;
use std::process::exit;
use std::sync::mpsc::{SyncSender, Receiver};
use std::thread::{self, sleep};
use std::time::Duration;

use closet::clone_army;
use gdk::{DisplayExt, EventMask};
use glib::markup_escape_text;
use gtk::prelude::*;
use gtk::{CssProvider, ScrolledWindow, self, StyleContext};
use structopt::StructOpt;

use crate::delay::Delay;
use crate::dictionary::{Definition, Dictionary, Entry, Text};



#[derive(StructOpt, Debug)]
#[structopt(name = "server-gui")]
pub struct Opt {
    #[structopt(short = "f", long = "font-name")]
    pub font_name: Option<String>,
    #[structopt(short = "s", long = "font-size")]
    pub font_size: Option<f64>,
}


pub fn main(tx: SyncSender<Option<Vec<Entry>>>, rx: Receiver<Option<Vec<Entry>>>, opt: Opt, dictionary_path: PathBuf) {
    gtk::init().unwrap();

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    WidgetExt::set_name(&window, "application");
    window.set_title("eitaro");
    window.set_role("eitaro");
    #[allow(deprecated)]
    window.set_wmclass("eitaro", "eitaro");
    window.set_border_width(0);
    // window.set_position(gtk::WindowPosition::Center);
    window.add_events(EventMask::SCROLL_MASK.bits() as i32);

    let scroller = ScrolledWindow::new(None, None);
    WidgetExt::set_name(&scroller, "scroller");
    window.add(&scroller);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 2);

    let label = gtk::Label::new(None);
    WidgetExt::set_name(&label, "label");
    label.set_line_wrap(true);
    label.set_selectable(true);
    vbox.pack_end(&label, true, true, 0);

    let entry = gtk::Entry::new();
    vbox.pack_end(&entry, false, false, 0);

    scroller.add(&vbox);

    let display = window.get_display().unwrap();
    let screen = display.get_default_screen();
    let css_provider = CssProvider::new();
    StyleContext::add_provider_for_screen(&screen, &css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    css_provider.load_from_data("#label { background-color: #004040; }".as_bytes()).unwrap();

    vbox.show();
    label.show();
    scroller.show();
    window.show();

    connect_events(window, &scroller, entry, dictionary_path, tx);

    let font_size = opt.font_size.unwrap_or(13.0);

    loop {
        while gtk::events_pending() {
            gtk::main_iteration();
        }

        for entries in rx.try_iter() {
            if let Some(entries) = entries {
                let mut content = format!(r#"<span font="{}""#, font_size);
                if let Some(font_name) = &opt.font_name {
                    write!(content, r#"face="{}""#, font_name).unwrap();
                }
                write!(content, ">").unwrap();
                markup_entries(&mut content, &entries);
                write!(content, "</span>").unwrap();
                label.set_markup(&content);
            }
        }

        sleep(Duration::from_millis(1));
    }
}

fn markup_entries(out: &mut String, entries: &[Entry]) {
    for entry in entries {
        color(out, &entry.key, "black", Some("yellow"), true);
        writeln!(out).unwrap();

        for definition in &entry.definitions {
            markup_definition(out, definition);
            writeln!(out).unwrap();
        }
    }
}

fn markup_definition(out: &mut String, definition: &Definition) {
    for (index, text) in definition.content.iter().enumerate() {
        if 0 < index {
            write!(out, " ").unwrap();
        }
        markup_text(out, text);
    }
}

fn markup_text(out: &mut String, text: &Text) {
    use self::Text::*;

    match &text {
        Annot(s) => color(out, s, "yellow", None, false),
        Countability(c) => color(out, &c.to_string(), "yellow", None, false),
        Class(s) => color(out, s, "lightblue", None, false),
        Definition(s) => color(out, s, "white", None, true),
        Error(s) => color(out, s, "red", None, true),
        Etymology(s) => {
            color(out, "語源 ", "magenta", None, true);
            color(out, s, "white", None, false);
        },
        Example(s) => color(out, s, "lightgreen", None, false),
        Information(s) => color(out, s, "cyan", None, false),
        Note(s) => color(out, s, "white", None, false),
        Tag(s) => color(out, s, "orangered", None, false),
        Word(s) => color(out, s, "black", Some("yellow"), false),
    }

}

fn color(out: &mut String, s: &str, fg: &str, bg: Option<&str>, bold: bool) {
    write!(out, r#"<span foreground="{}""#, fg).unwrap();
    if let Some(bg) = bg {
        write!(out, r#" background="{}""#, bg).unwrap();
    }
    if bold {
        write!(out, r#" weight="bold""#).unwrap();
    }
    write!(out, r#">{}</span>"#, markup_escape_text(s)).unwrap();
}

fn connect_events(window: gtk::Window, scroller: &gtk::ScrolledWindow, entry: gtk::Entry, dictionary_path: PathBuf, tx: SyncSender<Option<Vec<Entry>>>) {
    let delay = Delay::new(Duration::from_millis(250));

    window.connect_delete_event(|_, _| {
        exit(0);
    });

    entry.connect_key_press_event(clone_army!([window, scroller] move |entry, ev| {
        let empty = entry.get_text().map(|it| it.is_empty()).unwrap_or(true);
        let key = to_key_string(&ev);
        match &*key {
            "Return" | "Escape" => {
                if empty {
                    entry.hide();
                    window.set_focus(Some(&scroller));
                } else {
                    entry.set_text("");
                }
                return Inhibit(true);
            },
            _ => (),
        }
        Inhibit(false)
    }));

    entry.connect_key_release_event(move |entry, _| {
        if let Some(query) = entry.get_text() {
            if query.is_empty() {
                return Inhibit(false);
            }
            if let Ok(entries) = Dictionary::get_word(&dictionary_path, &query) {
                thread::spawn(clone_army!([tx, delay] move || {
                    if delay.wait() {
                        tx.send(entries).unwrap()
                    }
                }));
            }
        }
        Inhibit(false)
    });

    scroller.connect_delete_event(|_, _| exit(0));

    scroller.connect_key_press_event(clone_army!([entry] move |scroller, ev| {
        let key = to_key_string(&ev);
        match &*key {
            "q" | "Escape" => exit(0),
            "j" | "Down" => scroll(&scroller, false),
            "k" | "Up" => scroll(&scroller, true),
            "slash" => {
                window.set_focus(Some(&entry));
                entry.show();
            },
            _ => (),
        }
        Inhibit(false)
    }));
}

fn scroll(window: &ScrolledWindow, up: bool) {
    if let Some(adj) = window.get_vadjustment() {
        let mut page_size = adj.get_page_size();
        if up {
            page_size *= -1.0;
        }
        adj.set_value(page_size + adj.get_value());
    }
}

fn to_key_string(ev: &gdk::EventKey) -> String {
    let keyval = ev.as_ref().keyval;
    gdk::keyval_name(keyval).unwrap_or_else(|| format!("{}", keyval))
}
