use props::{Properties, WriteOption};

fn main() {
    let mut buff = Vec::new();

    let mut prop = Properties::new();
    prop.set("Hello", "World");
    prop.set("Hey", "你好©🌐");

    if let Err(e) = prop.store(&mut buff, &WriteOption::default()) {
        println!("Store properties failed, {}", e);
        return;
    }
    print!("{}", String::from_utf8(buff).unwrap());
}
