//! Provides a function to append a YAML doc to a file.
use std::io::Write;
use std::{fs::File, path::Path};

use serde::Serialize;

/// Appends serialized YAML data to a file, creating the file if it doesn't exist.
///
/// If the file already exists and contains data, a `---` separator is added before
/// appending the new data.  This allows for multiple YAML documents to be stored within a single file.
///
/// # Arguments
///
/// * `path` - The path to the file.
/// * `t` - The data to serialize and append, which must implement the `Serialize` trait from `serde`.
///
/// # Returns
///
/// * `Ok(())` if the operation is successful.
/// * An error if the file cannot be opened, written to, or the serialization fails.
///
/// # Examples
///
/// ```rust
/// use std::path::Path;
/// use serde::Serialize;
/// use syt::append::append_or_new;
/// use syt::Error;
/// use tempfile::NamedTempFile;
///
/// #[derive(Serialize)]
/// struct MyData {
///     name: String,
///     value: i32,
/// }
///
/// # fn main() -> Result<(), Error> {
/// let mut file = NamedTempFile::new()?;
/// let path = file.path();
///
/// let data1 = MyData { name: "first".to_string(), value: 1 };
/// append_or_new(&path, data1)?;
///
/// let data2 = MyData { name: "second".to_string(), value: 2 };
/// append_or_new(&path, data2)?;
///
/// // Resulting my_data.yml file:
/// // name: first
/// // value: 1
/// // ---
/// // name: second
/// // value: 2
/// # Ok(())
/// # }
/// ```
pub fn append_or_new<T: Serialize>(path: &Path, t: T) -> crate::Result<()> {
    let mut file = File::options().append(true).create(true).open(path)?;
    let metadata = file.metadata()?;
    if metadata.len() != 0 {
        file.write_all(b"\n---\n")?;
    }
    serde_yml::ser::to_writer(file, &t)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use std::fs::{self, File};
    use std::io::Read;
    use std::os::unix::fs::PermissionsExt;

    use tempfile::NamedTempFile;

    use serde::{Deserialize, Serialize};

    use crate::append::append_or_new;
    use crate::lazy::LazyDocs;

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct TestData {
        a: i32,
        b: String,
    }

    #[test]
    fn append_to_empty_file() -> crate::Result<()> {
        // GIVEN
        let tmp_file = NamedTempFile::new()?;
        let path = tmp_file.path();
        let data = TestData {
            a: 1,
            b: "hello".to_string(),
        };

        // WHEN
        append_or_new(path, &data)?;

        // THEN
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        assert_eq!(contents, "a: 1\nb: hello\n");
        Ok(())
    }

    #[test]
    fn append_to_existing_file() -> crate::Result<()> {
        // GIVEN
        let tmp_file = NamedTempFile::new()?;
        let path = tmp_file.path();
        let initial_data = TestData {
            a: 2,
            b: "world".to_string(),
        };
        serde_yml::ser::to_writer(File::create(path)?, &initial_data)?;
        let new_data = TestData {
            a: 1,
            b: "hello".to_string(),
        };

        // WHEN
        append_or_new(path, &new_data)?;

        // THEN
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        assert_eq!(contents, "a: 2\nb: world\n\n---\na: 1\nb: hello\n");
        Ok(())
    }

    #[test]
    fn create_file_if_not_exists() -> crate::Result<()> {
        // GIVEN
        let tmp_dir = tempfile::tempdir()?;
        let path = tmp_dir.path().join("new_file.yml");
        let data = TestData {
            a: 1,
            b: "hello".to_string(),
        };

        // WHEN
        append_or_new(&path, &data)?;

        // THEN
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        assert_eq!(contents, "a: 1\nb: hello\n");

        Ok(())
    }

    // Test for handling potential errors
    #[test]
    fn handle_write_error() {
        // GIVEN - A read-only directory (should cause write error)
        let tmp_dir = tempfile::tempdir().unwrap();
        let read_only_dir = tmp_dir.path().join("read_only");
        fs::create_dir(&read_only_dir).unwrap();
        let permissions = fs::Permissions::from_mode(0o444); // Read-only
        fs::set_permissions(&read_only_dir, permissions).unwrap();

        let path = read_only_dir.join("test.yml");
        let data = TestData {
            a: 1,
            b: "hello".to_string(),
        };

        // WHEN
        let result = append_or_new(&path, &data);

        // THEN
        assert!(result.is_err());
    }

    #[test]
    fn append_to_empty_file_and_lazy_load() -> crate::Result<()> {
        // GIVEN tmp file
        let tmp_file = NamedTempFile::new()?;
        let path = tmp_file.path();
        let data = TestData {
            a: 1,
            b: "hello".to_string(),
        };

        // WHEN append
        append_or_new(path, &data)?;

        // THEN read doc 1 and it equals expected
        let mut docs = LazyDocs::<TestData>::new(path).unwrap();
        let actual = docs.next();
        assert_eq!(
            actual,
            Some(TestData {
                a: 1,
                b: "hello".to_string()
            })
        );
        Ok(())
    }

    #[test]
    fn append_to_new_file_twice_and_lazy_load_docs() -> crate::Result<()> {
        // GIVEN tmp file
        let tmp_file = NamedTempFile::new()?;
        let path = tmp_file.path();

        // WHEN append data
        let initial_data = TestData {
            a: 2,
            b: "world".to_string(),
        };
        append_or_new(path, &initial_data)?;

        // WHEN append more data
        let new_data = TestData {
            a: 1,
            b: "hello".to_string(),
        };
        append_or_new(path, &new_data)?;

        // THEN first doc is initial data
        let mut docs = LazyDocs::<TestData>::new(path).unwrap();
        let actual = docs.next();
        assert_eq!(actual, Some(initial_data));

        // THEN second doc is new data
        let actual = docs.next();
        assert_eq!(actual, Some(new_data));
        Ok(())
    }
}
