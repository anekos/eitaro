
use std::fs::OpenOptions;
use std::fs::{read_dir, File};
use std::io::{Read, Write};



fn main() {
    let mut ups = vec![];

    for it in read_dir("migrations").unwrap() {
        let it = it.unwrap();
        if it.metadata().unwrap().is_dir() {
            let mut path = it.path().to_path_buf();
            path.push("up.sql");
            ups.push(path);
        }
    }

    ups.sort();

    let mut mig = OpenOptions::new().write(true).create(true).truncate(true).open("migrations.sql").unwrap();

    for up in ups {
        let mut file = File::open(up).unwrap();
        let mut sql = "".to_owned();
        file.read_to_string(&mut sql).unwrap();
        writeln!(mig, "{}", sql).unwrap();
    }
    writeln!(mig, "SELECT true").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
}
