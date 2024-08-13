# Java Properties for Rust

This is a library for readding a

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
argument. In most case, it will be UTF-8.

