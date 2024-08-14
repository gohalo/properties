use foobar::properties::Properties;

fn main() {
    //let f = "a0=你好🌐".as_bytes();
    //let f = "a0=b\\:b".as_bytes();
    //let f = "a0=\\u4r60\\u597d\\u00a9\\ud83c\\udf10\r\n".as_bytes();
    let f = "a0=\\u0009".as_bytes();

    let mut prop = Properties::new();
    match prop.load(f) {
        Ok(_) => {
            let v = prop.get("a0").unwrap();
            println!(
                "Load properties success, hello={:?} --> {}",
                v.as_bytes(),
                v
            )
        }
        Err(e) => println!("Load properties failed, {}", e),
    }
}
