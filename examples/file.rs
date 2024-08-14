/*
use std::collections::HashMap;
use std::io::BufWriter;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
*/

fn main() {
    println!("-----> {}", '你' as u32);

    let s1 = "你好🌐";
    println!("{:?}", s1.as_bytes());
    let s2 = "\u{4f60}\u{597d}\u{1f310}";
    println!("{:?}", s2.as_bytes());
    println!("{}", s2);

    /*
    let f = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .open("/tmp/test.properties")
        .unwrap();
    */

    /*
    //let f = File::open("/home/andy/Documents/rust/data/example/.hoodie/hoodie.properties").unwrap();
    let f = "a1=    bxxx".as_bytes();
    match prop.load(f) {
        Ok(_) => log::info!("Load properties success."),
        Err(e) => log::error!("Load properties failed, {}", e),
    }
    log::info!("--------> {:?}", prop.get("a1").unwrap());
    */

    /*
    let f = File::open("/home/andy/Documents/rust/data/example/.hoodie/hoodie.properties").unwrap();
    let mut reader = BufReader::new(f);
    let mut line = String::new();
    let len = reader.read_line(&mut line).unwrap();
    println!("First line is {len}");

    let mut prop = HashMap::new();
    prop.insert("Hello", "World");
    let mut file = File::create("/tmp/test.properties").unwrap();
    write!(BufWriter::new(file), &prop).unwrap();
    let mut prop = Properties::new();
    prop.set("Hello", "World");
    println!("Hello = {}", prop.get("Hello"));
    */
}
