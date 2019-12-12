
use std::fmt::Write;
use std::process::exit;
use std::sync::mpsc::{SyncSender, Receiver};
use std::thread::sleep;
use std::time::Duration;

use gdk::{DisplayExt, EventMask};
use glib::markup_escape_text;
use gtk::prelude::*;
use gtk::{CssProvider, ScrolledWindow, self, StyleContext};

use crate::dictionary::{Definition, Entry, Text};



#[derive(Clone)]
pub struct Gui {
    tx: SyncSender<Option<Vec<Entry>>>
}

pub fn main(rx: Receiver<Option<Vec<Entry>>>, font_name: Option<String>, font_size: f64) {
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

    let result_label = gtk::Label::new(None);
    WidgetExt::set_name(&result_label, "label");
    result_label.set_line_wrap(true);
    result_label.set_selectable(true);
    scroller.add(&result_label);

    let display = window.get_display().unwrap();
    let screen = display.get_default_screen();
    let css_provider = CssProvider::new();
    StyleContext::add_provider_for_screen(&screen, &css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

    css_provider.load_from_data("#label { background-color: #004040; }".as_bytes()).unwrap();

    connect_events(&scroller);
    window.show_all();

    loop {
        while gtk::events_pending() {
            gtk::main_iteration();
        }

        for entries in rx.try_iter() {
            if let Some(entries) = entries {
                let mut content = format!(r#"<span font="{}""#, font_size);
                if let Some(font_name) = &font_name {
                    write!(content, r#"face="{}""#, font_name).unwrap();
                }
                write!(content, ">").unwrap();
                markup_entries(&mut content, &entries);
                write!(content, "</span>").unwrap();
                result_label.set_markup(&content);
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
        Class(s) => color(out, s, "blue", None, false),
        Definition(s) => color(out, s, "white", None, true),
        Etymology(s) => {
            color(out, "語源 ", "magenta", None, true);
            color(out, s, "white", None, false);
        },
        Example(s) => color(out, s, "green", None, false),
        Information(s) => color(out, s, "cyan", None, false),
        Note(s) => color(out, s, "white", None, false),
        Tag(s) => color(out, s, "red", None, false),
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

fn connect_events(window: &gtk::ScrolledWindow) {
    window.connect_delete_event(|_, _| exit(0));

    window.connect_key_press_event(|scroller, ev| {
        let keyval = ev.as_ref().keyval;
        // let mut key = get_modifiers_text(ev.get_state(), true);
        let key: String = gdk::keyval_name(keyval).unwrap_or_else(|| format!("{}", keyval));
        match &*key {
            "q" | "Escape" => exit(0),
            "j" | "Down" => scroll(&scroller, false),
            "k" | "Up" => scroll(&scroller, true),
            _ => (),
        }
        Inhibit(false)
    });
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
