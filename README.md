# syt

This crate provides "things" for [serde_yml] or "serde_yml" things. It is mostly a bunch of hacks
consisting of the following:

* Functions to append YAML documents to a YAML file.
* An iterator to lazy load multiple YAML docs from the same file.
* A writer that inserts YAML comments based on a callback.


## Example of appending and lazy load YAML docs


```rust
use std::io::Write;
use tempfile::NamedTempFile;
use serde::Deserialize;
use serde::Serialize;
use syt::lazy::LazyDocs;
use syt::Error;
use syt::append::append_or_new;

/// Example Docs
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
enum MyDoc {
    Variant1 {
        title: String,
        content: String,
    },
    Variant2 {
        id: u32,
        data: Vec<String>,
    }
}

fn main() -> Result<(), Error> {

    let mut file = NamedTempFile::new()?;
    let path = file.path();

    let doc1 = MyDoc::Variant1 {
        title: "Doc 1".to_string(),
        content: "This is the first document.".to_string(),
    };
    // Example usage of append_or_new
    append_or_new(path, &doc1)?;

    let doc2 = MyDoc::Variant2 {
        id: 123,
        data: vec!["Item 1".to_string(), "Item 2".to_string()],
    };
    // Example usage of append_or_new
    append_or_new(path, &doc2)?;

    // Example usage of LazyDocs
    let docs = LazyDocs::<MyDoc>::new(path)?;

    let mut doc_iter = docs.into_iter();

    assert_eq!(doc_iter.next(), Some(doc1));
    assert_eq!(doc_iter.next(), Some(doc2));
    assert_eq!(doc_iter.next(), None);
    Ok(())
}
```

# Example of adding comments to YAML docs.

```rust
use serde::Serialize;
use syt::comments::{to_string, KeyData};
#[derive(Serialize)]
struct Config {
    name: String,
    age: u32,
}

let config = Config {
    name: "John Doe".to_string(),
    age: 30,
};

let cb = |key: KeyData| {
    if key.str == "name" {
        Some("The name of the person.".to_string())
    } else if key.str == "age" {
        Some("The age of the person.\nIn years.".to_string())
    } else {
        None
    }
};

let result = to_string(&config, cb).unwrap();

let expected = "\
    ## The name of the person.\n\
    name: John Doe\n\
    ## The age of the person.\n\
    ## In years.\n\
    age: 30\n\
    "
.trim_start()
.to_string();
assert_eq!(result, expected);
```

## License

Licensed under either of

* Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT) at your option.
