use std::borrow::Cow;

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
    if specifier.is_empty() {
      return Err(SpecifierError::Empty(specifier.to_string()));
    }
    let offset = match specifier.as_bytes()[0] {
      b'/' | b'.' | b'#' => 1,
      _ => 0,
    };
    let (path, query, fragment) = Self::parse_query_framgment(specifier, offset);
    if path.is_empty() {
      return Err(SpecifierError::Empty(specifier.to_string()));
    }
    Ok(Self {
      path,
      query,
      fragment,
    })
  }

  fn parse_query_framgment(
    specifier: &'a str,
    skip: usize,
  ) -> (Cow<'a, str>, Option<&'a str>, Option<&'a str>) {
    let mut query_start: Option<usize> = None;
    let mut fragment_start: Option<usize> = None;
    let mut escaped_indexes: Vec<usize> = Vec::new();

    // Scan as bytes: `?`, `#`, and `\0` are single-byte ASCII (< 0x80), so their
    // byte positions coincide with char positions in any UTF-8 input. This skips
    // the per-step UTF-8 decode that `char_indices()` performs, which dominated
    // the callgrind baseline.
    let bytes = specifier.as_bytes();
    let mut prev = bytes[0];
    for (i, &b) in bytes.iter().enumerate().skip(skip) {
      if b == b'?' && query_start.is_none() {
        query_start = Some(i);
      }
      if b == b'#' {
        if prev == 0 {
          escaped_indexes.push(i - 1);
        } else {
          fragment_start = Some(i);
          break;
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

    let path = if escaped_indexes.is_empty() {
      Cow::Borrowed(path)
    } else {
      // Each escaped index points at a `\0` byte that we need to drop. Copy the
      // surrounding slices in one pass — O(n) — instead of re-decoding chars and
      // calling `escaped_indexes.contains(&i)` per char, which was O(n*k).
      // The slice indices land on char boundaries because `\0` is ASCII.
      let mut s = String::with_capacity(path.len() - escaped_indexes.len());
      let mut last = 0;
      for &esc in &escaped_indexes {
        s.push_str(&path[last..esc]);
        last = esc + 1;
      }
      s.push_str(&path[last..]);
      Cow::Owned(s)
    };

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
