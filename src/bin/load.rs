use foobar::properties::Properties;

fn main() {
    env_logger::init();

    let f = "hello=你好🌐".as_bytes();

    let mut prop = Properties::new();
    match prop.load(f) {
        Ok(_) => println!(
            "Load properties success, hello={}",
            prop.get("hello").unwrap()
        ),
        Err(e) => println!("Load properties failed, {}", e),
    }
}
