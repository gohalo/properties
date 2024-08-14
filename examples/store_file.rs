use std::fs::OpenOptions;
use std::io::BufWriter;

use props::{Properties, WriteOption};

fn main() {
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open("/tmp/test.properties")
        .unwrap();
    let mut buff = BufWriter::new(file);

    let mut prop = Properties::new();
    prop.set("Hello", "World");
    prop.set("Hey", "你好©🌐");

    if let Err(e) = prop.store(&mut buff, &WriteOption::default()) {
        println!("Store properties failed, {}", e);
        return;
    }
}
