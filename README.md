# Java Properties for Rust

This is a library for readding a

# Encoding/Decoding

The properties is encoded in ISO 8859-1 character encoding. Characters that
cannot be directly represented in this encoding can be written using Unicode
escapes.




```
use datafusion::arrow::array::{Int32Array, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::dataframe::DataFrameWriteOptions;

    let ctx2 = SessionContext::new();
    let schema = Arc::new(Schema::new(vec![
        Field::new("a", DataType::Utf8, false),
        Field::new("b", DataType::Int32, false),
    ]));
    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(StringArray::from(vec!["a"])),
            Arc::new(Int32Array::from(vec![1])),
        ],
    )?;
    ctx2.register_batch("foobar", batch)?;
    let df2 = ctx2.table("foobar").await?;
    //df2.show().await?;
    df2.write_table("foobar", DataFrameWriteOptions::new())
        .await?;
    //ctx.sql("INSERT INTO example SELECT * from example").await?;
```
