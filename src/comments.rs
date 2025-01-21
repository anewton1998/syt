//! Code for adding comments to YAML docs.
use std::io::{self, Write};

use serde::Serialize;

/// Serializes a serializable value to a writer with comments.
///
/// This function takes a serializable value, a writer, and a callback function.
/// It serializes the value to the writer in YAML format, using the callback function to
/// associate comments with specific keys.
///
/// # Limitations
///
/// This is an ugly hack that works by wrapping the a [Write] object and scanning for something
/// that looks like a YAML key name. It does not account for quoted key names or escaping in the
/// key names and likely other YAML corner cases.
///
/// # Arguments
///
/// * `writer` - The writer to serialize the YAML to. Must implement the `Write` trait.
/// * `value` - A reference to the value to serialize.  Must implement the `Serialize` trait.
/// * `cb` - A callback function that takes a [`KeyData`] argument and returns an optional string.
///   If a string is returned, it will be used as a comment for the corresponding key.  The comment
///   can contain multiple lines separated by newline characters (`\n`).
///   Empty lines in the comment will be rendered as comment lines.
///   The `KeyData` provides the name of the key and its starting position.
///
/// # Returns
///
/// * `Ok(())` - If serialization was successful.
/// * `Err(crate::Error)` - An error if serialization fails.
///
/// # Example
///
/// ```
/// use serde::Serialize;
/// use std::io::Cursor;
/// use syt::comments::{to_writer, KeyData};
///
/// #[derive(Serialize)]
/// struct Config {
///     name: String,
///     age: u32,
/// }
///
/// let config = Config {
///     name: "John Doe".to_string(),
///     age: 30,
/// };
///
/// let mut writer = Cursor::new(Vec::new());
///
/// let cb = |key: KeyData| {
///     if key.str == "name" {
///         Some("The name of the person.".to_string())
///     } else if key.str == "age" {
///         Some("The age of the person.\nIn years.".to_string())
///     } else {
///         None
///     }
/// };
///
/// to_writer(&mut writer, &config, cb).unwrap();
///
/// let result = String::from_utf8(writer.into_inner()).unwrap();
///
/// let expected = "\
///     ## The name of the person.\n\
///     name: John Doe\n\
///     ## The age of the person.\n\
///     ## In years.\n\
///     age: 30\n\
///     "
/// .trim_start()
/// .to_string();
/// assert_eq!(result, expected);
/// ```
pub fn to_writer<W, T, F>(writer: W, value: &T, cb: F) -> crate::Result<()>
where
    W: Write,
    T: ?Sized + Serialize,
    F: Fn(KeyData) -> Option<String>,
{
    let commenter = Commenter::new(writer, cb);
    serde_yml::ser::to_writer(commenter, value)?;
    Ok(())
}

/// Serializes a serializable value to a YAML string with comments.
///
/// This function takes a serializable value and a callback function and produces a YAML string.
/// The callback function is invoked for each key in the serialized output, allowing you to
/// associate comments with specific keys.
///
/// # Limitations
///
/// This is an ugly hack that works by wrapping the a [Write] object and scanning for something
/// that looks like a YAML key name. It does not account for quoted key names or escaping in the
/// key names and likely other YAML corner cases.
///
/// # Arguments
///
/// * `value` - A reference to the value to serialize.  Must implement the `Serialize` trait.
/// * `cb` - A callback function that takes a [`KeyData`] argument and returns an optional string.
///   If a string is returned, it will be used as a comment for the corresponding key.  The comment
///   can contain multiple lines separated by newline characters (`\n`).
///   Empty lines in the comment will be rendered as empty lines.
///   The `KeyData` provides the name of the key.
///
/// # Returns
///
/// * `Ok(String)` - The serialized YAML string with comments.
/// * `Err(crate::Error)` - An error if serialization fails or if the resulting byte vector is not valid UTF-8.
///
/// # Example
///
/// ```
/// use serde::Serialize;
/// use syt::comments::{to_string, KeyData};
///
/// #[derive(Serialize)]
/// struct Config {
///     name: String,
///     age: u32,
/// }
///
/// let config = Config {
///     name: "John Doe".to_string(),
///     age: 30,
/// };
///
/// let cb = |key: KeyData| {
///     if key.str == "name" {
///         Some("The name of the person.".to_string())
///     } else if key.str == "age" {
///         Some("The age of the person.\nIn years.".to_string())
///     } else {
///         None
///     }
/// };
///
/// let result = to_string(&config, cb).unwrap();
///
/// let expected = "\
///     ## The name of the person.\n\
///     name: John Doe\n\
///     ## The age of the person.\n\
///     ## In years.\n\
///     age: 30\n\
///     "
/// .trim_start()
/// .to_string();
/// assert_eq!(result, expected);
/// ```
pub fn to_string<T, F>(value: &T, cb: F) -> crate::Result<String>
where
    T: ?Sized + Serialize,
    F: Fn(KeyData) -> Option<String>,
{
    let mut vec = Vec::with_capacity(128);
    to_writer(&mut vec, value, cb)?;
    let s = String::from_utf8(vec)?;
    Ok(s)
}

/// A writer wrapper that adds comments to YAML output.
///
/// This struct wraps a writer and intercepts the serialized YAML output.
/// It uses a callback function to determine which keys should have comments
/// and inserts the comments before the corresponding keys in the output.
///
/// # Type Parameters
///
/// * `W` - The underlying writer type. Must implement the `Write` trait.
/// * `F` - The callback function type.  Takes a [`KeyData`] argument and returns an optional string.
///
/// # Example
///
/// ```
/// use serde::Serialize;
/// use std::io::Cursor;
/// use syt::comments::{Commenter, KeyData};
///
/// #[derive(Serialize)]
/// struct Config {
///     name: String,
///     age: u32,
/// }
///
/// let config = Config {
///     name: "John Doe".to_string(),
///     age: 30,
/// };
///
/// let mut writer = Cursor::new(Vec::new());
///
/// let cb = |key: KeyData| {
///     if key.str == "name" {
///         Some("The name of the person.".to_string())
///     } else if key.str == "age" {
///         Some("The age of the person.\nIn years.".to_string())
///     } else {
///         None
///     }
/// };
///
/// let commenter = Commenter::new(writer, cb);
/// serde_yml::ser::to_writer(commenter, &config).unwrap();
/// // ... process the output from the writer ...
/// ```
pub struct Commenter<W, F>
where
    W: Write,
    F: Fn(KeyData) -> Option<String>,
{
    inner: W,
    cb: F,
    buffer: String,
}

impl<W, F> Commenter<W, F>
where
    W: Write,
    F: Fn(KeyData) -> Option<String>,
{
    pub fn new(writer: W, cb: F) -> Self {
        Commenter {
            inner: writer,
            cb,
            buffer: String::new(),
        }
    }

    fn flush_buffer(&mut self) -> io::Result<()> {
        if !self.buffer.is_empty() {
            if let Some(key) = get_key_name(&self.buffer) {
                let spacer_width = key.start;
                println!("key data: {key:?}");
                if let Some(s) = (self.cb)(key) {
                    for line in s.lines() {
                        let spacer = " ".repeat(spacer_width);
                        if line.is_empty() {
                            self.inner.write_fmt(format_args!("{spacer}\n"))?;
                        } else {
                            self.inner.write_fmt(format_args!("{spacer}# {line}\n"))?;
                        }
                    }
                }
            }
            self.inner.write_all(self.buffer.as_bytes())?;
            self.buffer.clear();
        }
        Ok(())
    }
}

impl<W, F> Write for Commenter<W, F>
where
    W: Write,
    F: Fn(KeyData) -> Option<String>,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        for c in s.chars() {
            self.buffer.push(c);
            if c == '\n' {
                self.flush_buffer()?;
            }
        }
        Ok(buf.len()) // claiming to have written everything
    }

    fn flush(&mut self) -> io::Result<()> {
        self.flush_buffer()?; // flush any remaining partial lines
        self.inner.flush() // flush the inner writer
    }
}

/// Key data information.
///
/// This struct holds the string representation of a key and its starting position
/// within a YAML document. It is used by the comment generation logic to associate
/// comments with specific keys.
#[derive(Debug, PartialEq, Eq)]
pub struct KeyData<'a> {
    /// The string representation of the key.
    pub str: &'a str,
    /// The starting byte position of the key within the YAML document.
    pub start: usize,
}

fn get_key_name(str: &str) -> Option<KeyData> {
    let mut start: Option<usize> = None;
    let mut end: Option<usize> = None;
    for (i, c) in str.char_indices() {
        if c.is_control() {
            continue;
        }
        if c == '#' && (start.is_none() || end.is_none()) {
            return None;
        }
        if (c == '-' || c == '?' || c.is_whitespace()) && start.is_none() {
            continue;
        }
        if c == ':' {
            if start.is_none() {
                return None;
            } else {
                end = Some(i - 1)
            }
        }
        if start.is_none() {
            start = Some(i);
        }
    }
    if start.is_some() && end.is_some() {
        let start = start.unwrap(); // checked above
        let end = end.unwrap(); // checked above
        let s = &str[start..=end];
        Some(KeyData { str: s, start })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_get_key_name() {
        assert_eq!(get_key_name("foo"), None);
        assert_eq!(
            get_key_name("foo:"),
            Some(KeyData {
                str: "foo",
                start: 0
            })
        );
        assert_eq!(
            get_key_name("  foo:"),
            Some(KeyData {
                str: "foo",
                start: 2
            })
        );
        assert_eq!(
            get_key_name("  foo bar:"),
            Some(KeyData {
                str: "foo bar",
                start: 2
            })
        );
        assert_eq!(
            get_key_name("- foo bar:"),
            Some(KeyData {
                str: "foo bar",
                start: 2
            })
        );
        assert_eq!(
            get_key_name("? foo bar:"),
            Some(KeyData {
                str: "foo bar",
                start: 2
            })
        );
    }

    #[test]
    fn test_to_string() {
        // GIVEN a simple struct
        #[derive(Serialize)]
        struct Config {
            name: String,
            age: u32,
        }

        let config = Config {
            name: "John Doe".to_string(),
            age: 30,
        };

        // GIVEN a callback function to add comments
        let cb = |key: KeyData| {
            if key.str == "name" {
                Some("The name of the person.".to_string())
            } else if key.str == "age" {
                Some("The age of the person.\nIn years.".to_string())
            } else {
                None
            }
        };

        // WHEN to_string
        let result = to_string(&config, cb).unwrap();

        // THEN expect comments in output
        let expected = "\
            # The name of the person.\n\
            name: John Doe\n\
            # The age of the person.\n\
            # In years.\n\
            age: 30\n\
            "
        .trim_start()
        .to_string();
        assert_eq!(result, expected);

        // WHEN to string with callback that does nothing
        let no_comments = to_string(&config, |_| None).unwrap();

        // THEN expect no comments in output
        let expected_no_comments = "
            name: John Doe\n\
            age: 30\n\
            "
        .trim_start()
        .to_string();
        assert_eq!(no_comments, expected_no_comments);
    }

    #[test]
    fn test_to_string_empty_struct() {
        // GIVEN a struct that has no no data
        #[derive(Serialize)]
        struct Empty {}

        let empty = Empty {};

        // WHEN to_string with callback that does nothing
        let result = to_string(&empty, |_| None).unwrap();

        // THEN nothing in the output
        assert_eq!(result, "{}\n");
    }

    #[test]
    fn test_to_string_nested() {
        // GIVEN a struct with an inner struct
        #[derive(Serialize)]
        struct Inner {
            value: String,
        }
        #[derive(Serialize)]
        struct Outer {
            inner: Inner,
        }

        let outer = Outer {
            inner: Inner {
                value: "hello".to_string(),
            },
        };

        // GIVEN a callback that recognizes the names
        let cb = |key: KeyData| {
            if key.str == "inner" {
                Some("inner struct".to_string())
            } else if key.str == "value" {
                Some("inner value".to_string())
            } else {
                None
            }
        };

        // WHEN to_string with the callback
        let result = to_string(&outer, cb).unwrap();

        // THEN expect comments for inner indented
        let expected = r#"
# inner struct
inner:
  # inner value
  value: hello
"#
        .trim_start()
        .to_string();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_string_round_trip() {
        // GIVEN a simple struct
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct Config {
            name: String,
            age: u32,
        }

        let config = Config {
            name: "John Doe".to_string(),
            age: 30,
        };

        // GIVEN a callback function to add comments
        let cb = |key: KeyData| {
            if key.str == "name" {
                Some("The name of the person.".to_string())
            } else if key.str == "age" {
                Some("The age of the person.\nIn years.".to_string())
            } else {
                None
            }
        };

        // WHEN to_string
        let result = to_string(&config, cb).unwrap();

        // WHEN parse string from to_string
        let actual = serde_yml::from_str::<Config>(&result).unwrap();

        // THEN actual should be the input
        assert_eq!(actual, config);
    }
}
