//! Provides a lazy iterator over YAML documents in a file, separated by "---".
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::marker::PhantomData;
use std::path::Path;

use serde::de::DeserializeOwned;

/// A lazy iterator over YAML documents in a file.
///
/// This struct reads a file line by line, parsing YAML documents delimited by "---".
/// It avoids loading the entire file into memory, making it suitable for large files.
/// Each YAML document is deserialized into a user-specified type `T`.
///
/// Example:
///
/// ```rust
/// use std::io::Write;
/// use tempfile::NamedTempFile;
/// use serde::Deserialize;
/// use syt::lazy::LazyDocs;
/// use syt::Error;
///
/// #[derive(Deserialize, Debug, PartialEq, Eq)]
/// struct MyDoc {
///     title: String,
///     content: String,
/// }
///
/// # fn main() -> Result<(), Error> {
/// let mut file = NamedTempFile::new()?;
/// writeln!(file, "---")?;
/// writeln!(file, "title: Doc 1")?;
/// writeln!(file, "content: This is the first document.")?;
/// writeln!(file, "---")?;
/// writeln!(file, "title: Doc 2")?;
/// writeln!(file, "content: This is the second document.")?;
/// writeln!(file, "---")?;
/// let path = file.path();
///
/// let docs = LazyDocs::<MyDoc>::new(path)?;
///
/// for doc in docs {
///    println!("Title: {}, Content: {}", doc.title, doc.content);
/// }
/// # Ok(())
/// # }
/// ```
pub struct LazyDocs<T: DeserializeOwned> {
    lines: Lines<BufReader<File>>,
    phatom: PhantomData<T>,
}

/// Creates a new `LazyDocs` iterator.
///
/// # Arguments
///
/// * `path` - The path to the YAML file.
///
/// # Errors
///
/// Returns an error if the file cannot be opened.
impl<T: DeserializeOwned> LazyDocs<T> {
    pub fn new(path: &Path) -> crate::Result<Self> {
        let file = File::open(path)?;
        let buf = BufReader::new(file);
        Ok(LazyDocs::<T> {
            lines: buf.lines(),
            phatom: PhantomData,
        })
    }
}

impl<T: DeserializeOwned> Iterator for LazyDocs<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = Vec::new();
        loop {
            let line = self.lines.next();
            if let Some(Ok(line)) = line {
                if line.starts_with("---") && !buf.is_empty() {
                    break;
                } else {
                    buf.push(line);
                }
            } else {
                break;
            }
        }
        if !buf.is_empty() {
            let s = buf.join("\n");
            serde_yml::from_str::<T>(&s).ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::Deserialize;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct TestDoc {
        title: String,
        content: String,
    }

    #[test]
    fn test_lazy_docs_iterator() {
        // GIVEN a file with two valid YAML documents separated by "---"
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "title: Doc 1").unwrap();
        writeln!(file, "content: This is the first document.").unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "title: Doc 2").unwrap();
        writeln!(file, "content: This is the second document.").unwrap();
        writeln!(file, "---").unwrap();
        let path = file.path();

        // WHEN creating a LazyDocs iterator
        let mut docs = LazyDocs::<TestDoc>::new(path).unwrap();

        // THEN it should yield the two documents correctly and then None
        let doc1 = docs.next();
        assert_eq!(
            doc1,
            Some(TestDoc {
                title: "Doc 1".to_string(),
                content: "This is the first document.".to_string()
            })
        );

        let doc2 = docs.next();
        assert_eq!(
            doc2,
            Some(TestDoc {
                title: "Doc 2".to_string(),
                content: "This is the second document.".to_string()
            })
        );

        assert!(docs.next().is_none());
    }

    #[test]
    fn test_lazy_docs_iterator_no_start_doc() {
        // GIVEN a file with two valid YAML documents separated by "---"
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "title: Doc 1").unwrap();
        writeln!(file, "content: This is the first document.").unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "title: Doc 2").unwrap();
        writeln!(file, "content: This is the second document.").unwrap();
        writeln!(file, "---").unwrap();
        let path = file.path();

        // WHEN creating a LazyDocs iterator
        let mut docs = LazyDocs::<TestDoc>::new(path).unwrap();

        // THEN it should yield the two documents correctly and then None
        let doc1 = docs.next();
        assert_eq!(
            doc1,
            Some(TestDoc {
                title: "Doc 1".to_string(),
                content: "This is the first document.".to_string()
            })
        );

        let doc2 = docs.next();
        assert_eq!(
            doc2,
            Some(TestDoc {
                title: "Doc 2".to_string(),
                content: "This is the second document.".to_string()
            })
        );

        assert!(docs.next().is_none());
    }

    #[test]
    fn test_lazy_docs_empty_file() {
        // GIVEN an empty file
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        // WHEN creating a LazyDocs iterator
        let mut docs = LazyDocs::<TestDoc>::new(path).unwrap();

        // THEN it should yield None
        assert!(docs.next().is_none());
    }

    #[test]
    fn test_lazy_docs_incomplete_yaml() {
        // GIVEN a file with an incomplete YAML document
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "title: Doc 1").unwrap();
        writeln!(file, "---").unwrap();
        let path = file.path();

        // WHEN creating a LazyDocs iterator
        let mut docs = LazyDocs::<TestDoc>::new(path).unwrap();

        // THEN it should skip the invalid YAML and yield None
        assert!(docs.next().is_none());
    }

    #[test]
    fn test_lazy_docs_no_separator() {
        // GIVEN a file with a valid YAML document but no "---" separator
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "title: Doc 1").unwrap();
        writeln!(file, "content: This is the first document.").unwrap();
        let path = file.path();

        // WHEN creating a LazyDocs iterator
        let mut docs = LazyDocs::<TestDoc>::new(path).unwrap();

        // THEN it should yield the document and then None
        let doc1 = docs.next();
        assert_eq!(
            doc1,
            Some(TestDoc {
                title: "Doc 1".to_string(),
                content: "This is the first document.".to_string()
            })
        );

        assert!(docs.next().is_none());
    }
}
