use properties::Properties;

fn main() {
    let buff = "hello=\\u4f60\\u597d\\u00a9\\ud83c\\udf10\nhey=你好🌐\n".as_bytes();

    let mut prop = Properties::new();
    match prop.load(buff) {
        Ok(_) => {
            println!("hello={}", prop.get("hello").unwrap());
            println!("hey={}", prop.get("hey").unwrap());
        }
        Err(e) => println!("Load properties failed, {}", e),
    }
}
