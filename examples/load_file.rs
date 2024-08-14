use std::fs::File;
use std::io::BufReader;

use properties::Properties;

fn main() {
    let file = File::open("/tmp/test.properties").unwrap();
    let reader = BufReader::new(file);

    let mut prop = Properties::new();
    if let Err(e) = prop.load(reader) {
        println!("Load properties failed, {}", e);
        return;
    }
    println!("{}", prop.get("Hey").unwrap());
}
