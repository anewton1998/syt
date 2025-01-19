//! Provides a lazy iterator over YAML documents in a file, separated by "---".
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::marker::PhantomData;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde_yml::Value;

/// A lazy iterator over YAML documents in a file.
///
/// This struct reads a file line by line, parsing YAML documents delimited by "---".
/// It avoids loading the entire file into memory, making it suitable for large files.
/// Each YAML document is deserialized into a user-specified type `T`.
///
/// Note that each YAML doc is read into memory before parsing, so if the docs themselves
/// are very large then this might not be as efficient as some may need.
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
///
/// Another Example using [crate::append::append_or_new]:
/// ```rust
/// use std::io::Write;
/// use tempfile::NamedTempFile;
/// use serde::Deserialize;
/// use serde::Serialize;
/// use syt::lazy::LazyDocs;
/// use syt::Error;
/// use syt::append::append_or_new;
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
/// enum MyDoc {
///     Variant1 {
///         title: String,
///         content: String,
///     },
///     Variant2 {
///         id: u32,
///         data: Vec<String>,
///     }
/// }
///
/// # fn main() -> Result<(), Error> {
/// let mut file = NamedTempFile::new()?;
/// let path = file.path();
///
/// let doc1 = MyDoc::Variant1 {
///     title: "Doc 1".to_string(),
///     content: "This is the first document.".to_string(),
/// };
/// append_or_new(path, &doc1)?;
///
/// let doc2 = MyDoc::Variant2 {
///     id: 123,
///     data: vec!["Item 1".to_string(), "Item 2".to_string()],
/// };
/// append_or_new(path, &doc2)?;
///
/// let docs = LazyDocs::<MyDoc>::new(path)?;
///
/// let mut doc_iter = docs.into_iter();
///
/// assert_eq!(doc_iter.next(), Some(doc1));
/// assert_eq!(doc_iter.next(), Some(doc2));
/// assert_eq!(doc_iter.next(), None);
/// # Ok(())
/// # }
/// ```
pub struct LazyDocs<T: DeserializeOwned> {
    lazy_values: LazyValues,
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
        Ok(LazyDocs::<T> {
            lazy_values: LazyValues::new(path)?,
            phatom: PhantomData,
        })
    }
}

impl<T: DeserializeOwned> Iterator for LazyDocs<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(v) = self.lazy_values.next() {
            serde_yml::from_value::<T>(v).ok()
        } else {
            None
        }
    }
}

/// A lazy iterator over YAML values in a file, separated by "---".
///
/// This struct reads a file line by line, parsing YAML documents delimited by "---".
/// It avoids loading the entire file into memory, making it suitable for large files.
/// Each YAML document is deserialized into a `serde_yml::Value`.  It is used by
/// `LazyDocs`, but can be used directly as well.
///
/// Example:
///
/// ```rust
/// use std::io::Write;
/// use tempfile::NamedTempFile;
/// use serde_yml::Value;
/// use syt::lazy::LazyValues;
/// use syt::Error;
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
/// let values = LazyValues::new(path)?;
///
/// for value in values {
///     println!("{:?}", value);
/// }
/// # Ok(())
/// # }
/// ```
pub struct LazyValues {
    doc_start: LazyDocStart,
}

impl LazyValues {
    /// Creates a new `LazyValues` iterator.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the YAML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened.
    pub fn new(path: &Path) -> crate::Result<Self> {
        Ok(LazyValues {
            doc_start: LazyDocStart::new(path)?,
        })
    }
}

impl Iterator for LazyValues {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = self.doc_start.next() {
            serde_yml::from_str(&s).ok()
        } else {
            None
        }
    }
}

/// A lazy iterator that yields strings representing YAML documents from a file.
///
/// This struct reads a file line by line, buffering lines until a "---" separator is encountered
/// or the end of the file is reached.  It returns a `String` representing the buffered lines
/// for each YAML document. Documents are separated by lines starting with "---".
/// Leading and trailing whitespace around the "---" separator are included in the document strings.
///
/// It avoids loading the entire file into memory, making it suitable for processing large files
/// containing multiple YAML documents.
///
/// Example:
///
/// ```rust
/// use std::io::Write;
/// use tempfile::NamedTempFile;
/// use syt::lazy::LazyDocStart;
/// use syt::Error;
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
/// let doc_starts = LazyDocStart::new(path)?;
///
/// for doc_str in doc_starts {
///     println!("{}", doc_str);
/// }
/// # Ok(())
/// # }
/// ```
pub struct LazyDocStart {
    lines: Lines<BufReader<File>>,
}

impl LazyDocStart {
    /// Creates a new `LazyDocStart` iterator.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the YAML file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened.
    pub fn new(path: &Path) -> crate::Result<Self> {
        let file = File::open(path)?;
        let buf = BufReader::new(file);
        Ok(LazyDocStart { lines: buf.lines() })
    }
}

impl Iterator for LazyDocStart {
    type Item = String;

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
            Some(buf.join("\n"))
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
