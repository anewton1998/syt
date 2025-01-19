# syt

This crate provides a function to append YAML documents to a YAML file, and to lazy load YAML
documents from files.

## Example


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


## License

Licensed under either of

* Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT) at your option.
