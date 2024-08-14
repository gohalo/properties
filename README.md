# Java Properties for Rust

This is a library for reading and writing Java properties in Rust.

* support UTF-8 and ISO 8859-1 encoding, including mixed (for reading).
* without any dependence, which also means no other encoding support, simple but enough.
* 100% compatible with java.
* almost 100% unit testing (not covered for some error code).
* support fuzz testing

# Examples

``` rust
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
```

And more examples under `examples` directory, you could simply run with `cargo run --example store` command.

# Encoding/Decoding

Java has different implements for `OutputStream` and `Writer`, which could
test from the following code.

``` java
import java.io.FileWriter;
import java.io.IOException;
import java.util.Properties;

public class Example {
    public static void main(String[] args) throws IOException {
        Properties p = new Properties();
        p.setProperty("hello", "你好🌐");

        // OutputStream(ISO 8859-1 encoding), hello=\u4F60\u597D\uD83C\uDF10
        p.store(System.out, null);

        // Writer(UTF-8 encoding), hello=你好🌐
        FileWriter w = new FileWriter("/tmp/test.properties");
        p.store(w, null);
        w.close();
    }
}
```

For `OutputStream`, the properties is encoded in ISO 8859-1 character encoding.
Characters that cannot be directly represented in this encoding can be written
using Unicode escapes.

And for `Writer`, it's determined by the file encoding or `-Dfile.encoding=xxx`
argument when start your program. In most case, it will be UTF-8.

So, in this library, support UTF-8 which is the encoding for Rust. And also support
ISO 8859-1 with the `escape_unicode` option.

# License

[Apache 2.0 @ GoHalo](./LICENSE)
