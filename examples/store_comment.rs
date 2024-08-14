use props::{Properties, WriteOption};

fn main() {
    let mut buff = Vec::new();
    let mut prop = Properties::new();

    let mut opt = WriteOption::default();
    //opt.escape_unicode(true);
    opt.comments("Hello\n你好©🌐".to_string());

    prop.set("Hello", "World");
    if let Err(e) = prop.store(&mut buff, &opt) {
        println!("Store properties failed, {}", e);
        return;
    }
    println!("{}", String::from_utf8(buff).unwrap());
}
