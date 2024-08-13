use foobar::properties::{Properties, WriteOption};

fn main() {
    env_logger::init();

    // Write to buffer.
    let mut buff = Vec::new();
    let mut prop = Properties::new();
    let mut opt = WriteOption::default();
    opt.escape_unicode(true);
    //opt.comments("Hello\n你好©🌐".to_string());
    //opt.comments("你好©🌐".to_string());
    opt.comments("Hello\r\n".to_string());

    //prop.set("Hello", "World");
    if let Err(e) = prop.store(&mut buff, &opt) {
        println!("Store properties failed, {}", e);
        return;
    }
    //println!("{:?}", buff);
    println!("{}", String::from_utf8(buff).unwrap());

    /*
    // Write to buffer.
    let mut buff = Vec::new();
    let mut prop = Properties::new();
    prop.set("Hello", "World");

    if let Err(e) = prop.store(&mut buff, &WriteOption::default()) {
        println!("Store properties failed, {}", e);
        return;
    }
    print!("Got result: {}", String::from_utf8(buff).unwrap());
    */

    /*
    let data = "你好©🌐";

    // Write to buffer.
    let mut buff = Vec::new();
    let mut prop = Properties::new();
    prop.set("Hello", data);
    println!("Got result: {:?}", data.as_bytes());

    if let Err(e) = prop.store(&mut buff, &WriteOption::default()) {
        println!("Store properties failed, {}", e);
        return;
    }
    print!("Got result: {}", String::from_utf8(buff).unwrap());
    //println!("Got result: {:?}", buff);
    */
}
