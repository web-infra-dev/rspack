use std::borrow::Cow;

#[cfg(windows)]
use rspack_paths::dos_device_path_prefix_len;

use crate::error::SpecifierError;

#[derive(Debug)]
pub struct Specifier<'a> {
  path: Cow<'a, str>,
  pub query: Option<&'a str>,
  pub fragment: Option<&'a str>,
}

impl<'a> Specifier<'a> {
  pub fn path(&'a self) -> &'a str {
    self.path.as_ref()
  }

  /// Parse a module specifier into path, query, and fragment.
  ///
  /// # Errors
  ///
  /// * See [SpecifierError]
  pub fn parse(specifier: &'a str) -> Result<Self, SpecifierError> {
    #[cold]
    fn empty_error(specifier: &str) -> SpecifierError {
      SpecifierError::Empty(specifier.to_string())
    }
    if specifier.is_empty() {
      return Err(empty_error(specifier));
    }
    #[cfg(windows)]
    let offset = {
      let dos_prefix_len = dos_device_path_prefix_len(specifier);
      if dos_prefix_len != 0 {
        dos_prefix_len
      } else {
        match specifier.as_bytes()[0] {
          b'/' | b'.' | b'#' => 1,
          _ => 0,
        }
      }
    };
    #[cfg(not(windows))]
    let offset = match specifier.as_bytes()[0] {
      b'/' | b'.' | b'#' => 1,
      _ => 0,
    };

    let (path, query, fragment) = Self::parse_query_fragment(specifier, offset);
    if path.is_empty() {
      return Err(empty_error(specifier));
    }
    Ok(Self {
      path,
      query,
      fragment,
    })
  }

  fn parse_query_fragment(
    specifier: &'a str,
    skip: usize,
  ) -> (Cow<'a, str>, Option<&'a str>, Option<&'a str>) {
    // Most specifiers carry no query or fragment. Resume at the first delimiter
    // when present so the stateful parser does not scan the path prefix again.
    let bytes = specifier.as_bytes();
    let Some(first) = memchr::memchr2(b'?', b'#', &bytes[skip..]) else {
      return (Cow::Borrowed(specifier), None, None);
    };
    let start = skip + first;

    let mut query_start: Option<usize> = None;
    let mut fragment_start: Option<usize> = None;

    let mut prev = bytes[start.saturating_sub(1)];
    // Optimize for the common case: most specifiers don't have escaped characters
    let mut escaped_indexes: Option<Vec<usize>> = None;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
      if b == b'?' && query_start.is_none() {
        query_start = Some(i);
      }
      if b == b'#' {
        if prev != b'\0' {
          fragment_start = Some(i);
          break;
        }
        // Query escapes remain in the query; only path escapes need copying.
        if query_start.is_none() {
          escaped_indexes.get_or_insert_with(Vec::new).push(i - 1);
        }
      }
      prev = b;
    }

    let (path, query, fragment) = match (query_start, fragment_start) {
      (Some(i), Some(j)) => {
        debug_assert!(i < j);
        (
          &specifier[..i],
          Some(&specifier[i..j]),
          Some(&specifier[j..]),
        )
      }
      (Some(i), None) => (&specifier[..i], Some(&specifier[i..]), None),
      (None, Some(j)) => (&specifier[..j], None, Some(&specifier[j..])),
      _ => (specifier, None, None),
    };

    let path = escaped_indexes.map_or(Cow::Borrowed(path), |escaped_indexes| {
      // NUL is ASCII, so each escaped index is a UTF-8 boundary. Copy the
      // surrounding slices once rather than searching the indexes per character.
      let mut s = String::with_capacity(path.len() - escaped_indexes.len());
      let mut last = 0;
      for esc in escaped_indexes {
        s.push_str(&path[last..esc]);
        last = esc + 1;
      }
      s.push_str(&path[last..]);
      Cow::Owned(s)
    });

    (path, query, fragment)
  }
}

#[cfg(test)]
mod tests {
  use super::{Specifier, SpecifierError};

  #[test]
  fn debug() {
    let specifier = Specifier::parse("/").unwrap();
    assert_eq!(
      format!("{specifier:?}"),
      r#"Specifier { path: "/", query: None, fragment: None }"#
    );
  }

  #[test]
  fn empty() {
    let specifiers = ["", "?"];
    for specifier in specifiers {
      let error = Specifier::parse(specifier).unwrap_err();
      assert_eq!(error, SpecifierError::Empty(specifier.to_string()));
    }
  }

  #[test]
  fn absolute() -> Result<(), SpecifierError> {
    let specifier = "/test?#";
    let parsed = Specifier::parse(specifier)?;
    assert_eq!(parsed.path, "/test");
    assert_eq!(parsed.query, Some("?"));
    assert_eq!(parsed.fragment, Some("#"));
    Ok(())
  }

  #[test]
  fn relative() -> Result<(), SpecifierError> {
    let specifiers = ["./test", "../test", "../../test"];
    for specifier in specifiers {
      let mut r = specifier.to_string();
      r.push_str("?#");
      let parsed = Specifier::parse(&r)?;
      assert_eq!(parsed.path, specifier);
      assert_eq!(parsed.query, Some("?"));
      assert_eq!(parsed.fragment, Some("#"));
    }
    Ok(())
  }

  #[test]
  fn hash() -> Result<(), SpecifierError> {
    let specifiers = ["#", "#path"];
    for specifier in specifiers {
      let mut r = specifier.to_string();
      r.push_str("?#");
      let parsed = Specifier::parse(&r)?;
      assert_eq!(parsed.path, specifier);
      assert_eq!(parsed.query, Some("?"));
      assert_eq!(parsed.fragment, Some("#"));
    }
    Ok(())
  }

  #[test]
  fn module() -> Result<(), SpecifierError> {
    let specifiers = ["module"];
    for specifier in specifiers {
      let mut r = specifier.to_string();
      r.push_str("?#");
      let parsed = Specifier::parse(&r)?;
      assert_eq!(parsed.path, specifier);
      assert_eq!(parsed.query, Some("?"));
      assert_eq!(parsed.fragment, Some("#"));
    }
    Ok(())
  }

  #[test]
  fn query_fragment() -> Result<(), SpecifierError> {
    let data = [
      ("a?", Some("?"), None),
      ("a?query", Some("?query"), None),
      ("a?query1?query2", Some("?query1?query2"), None),
      (
        "a?query1?query2?query3",
        Some("?query1?query2?query3"),
        None,
      ),
      ("a#", None, Some("#")),
      ("a#b#c", None, Some("#b#c")),
      ("a#fragment", None, Some("#fragment")),
      ("a?#", Some("?"), Some("#")),
      ("a?#fragment", Some("?"), Some("#fragment")),
      ("a?query#", Some("?query"), Some("#")),
      ("a?query#fragment", Some("?query"), Some("#fragment")),
      ("a#fragment?", None, Some("#fragment?")),
      ("a#fragment?query", None, Some("#fragment?query")),
    ];

    for (specifier_str, query, fragment) in data {
      let specifier = Specifier::parse(specifier_str)?;
      assert_eq!(specifier.path, "a", "{specifier_str}");
      assert_eq!(specifier.query, query, "{specifier_str}");
      assert_eq!(specifier.fragment, fragment, "{specifier_str}");
    }

    Ok(())
  }

  #[test]
  // https://github.com/webpack/enhanced-resolve/blob/main/test/identifier.test.js
  fn enhanced_resolve_edge_cases() -> Result<(), SpecifierError> {
    let data = [
      ("path/#", "path/", "", "#"),
      ("path/as/?", "path/as/", "?", ""),
      ("path/#/?", "path/", "", "#/?"),
      ("path/#repo#hash", "path/", "", "#repo#hash"),
      ("path/#r#hash", "path/", "", "#r#hash"),
      ("path/#repo/#repo2#hash", "path/", "", "#repo/#repo2#hash"),
      ("path/#r/#r#hash", "path/", "", "#r/#r#hash"),
      (
        "path/#/not/a/hash?not-a-query",
        "path/",
        "",
        "#/not/a/hash?not-a-query",
      ),
    ];

    for (specifier_str, path, query, fragment) in data {
      let specifier = Specifier::parse(specifier_str)?;
      assert_eq!(specifier.path, path, "{specifier_str}");
      assert_eq!(specifier.query.unwrap_or(""), query, "{specifier_str}");
      assert_eq!(
        specifier.fragment.unwrap_or(""),
        fragment,
        "{specifier_str}"
      );
    }

    Ok(())
  }

  // https://github.com/webpack/enhanced-resolve/blob/main/test/identifier.test.js
  #[test]
  fn enhanced_resolve_windows_like() -> Result<(), SpecifierError> {
    let data = [
      ("path\\#", "path\\", "", "#"),
      ("path\\as\\?", "path\\as\\", "?", ""),
      ("path\\#\\?", "path\\", "", "#\\?"),
      ("path\\#repo#hash", "path\\", "", "#repo#hash"),
      ("path\\#r#hash", "path\\", "", "#r#hash"),
      (
        "path\\#repo\\#repo2#hash",
        "path\\",
        "",
        "#repo\\#repo2#hash",
      ),
      ("path\\#r\\#r#hash", "path\\", "", "#r\\#r#hash"),
      (
        "path\\#/not/a/hash?not-a-query",
        "path\\",
        "",
        "#/not/a/hash?not-a-query",
      ),
    ];

    for (specifier_str, path, query, fragment) in data {
      let specifier = Specifier::parse(specifier_str)?;
      assert_eq!(specifier.path, path, "{specifier_str}");
      assert_eq!(specifier.query.unwrap_or(""), query, "{specifier_str}");
      assert_eq!(
        specifier.fragment.unwrap_or(""),
        fragment,
        "{specifier_str}"
      );
    }

    Ok(())
  }
}
