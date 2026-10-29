use std::fmt::Write;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use url::Host;

/// The WHATWG path segment set that `url::Url::from_file_path` applies to Windows components.
const PATH_SEGMENT: &AsciiSet = &CONTROLS
  .add(b' ')
  .add(b'"')
  .add(b'<')
  .add(b'>')
  .add(b'`')
  .add(b'#')
  .add(b'?')
  .add(b'{')
  .add(b'}')
  .add(b'/')
  .add(b'%');

/// The set `url::Url::from_file_path` applies to Unix components, where `\` is not a separator.
const SPECIAL_PATH_SEGMENT: &AsciiSet = &PATH_SEGMENT.add(b'\\');

/// Byte patterns under which `root` can appear in serialized cache data.
///
/// Detection is best-effort: it covers the spellings that path-producing code commonly emits, not
/// every possible transformation of the root (base64, other escapes, hashes and values computed
/// from it are not found). For each native spelling of the root, and for a Windows root also its
/// forward-slash spelling and both drive-letter cases, the patterns cover:
///
/// - the spelling itself, and with every `/` escaped as `\/` (JSON's escaped solidus, regex source);
/// - its `JSON.stringify` string content, once and nested twice;
/// - the path part of its `file://` URL, with a UNC host normalized the way
///   `url::Url::from_file_path` normalizes it.
///
/// The input is read as a Windows path when it starts with a drive letter or `\\`, on every host
/// platform. Trailing separators are ignored. A pattern that contains another pattern is dropped,
/// since any match of it is also a match of the shorter one.
pub fn root_patterns(root: &str) -> Vec<Vec<u8>> {
  let windows = is_windows_root(root);
  let trimmed = root.trim_end_matches(|c| c == '/' || (windows && c == '\\'));
  let root = if trimmed.is_empty() { root } else { trimmed };

  let mut spellings = vec![root.to_string()];
  if windows {
    spellings.push(replace_char(root, '\\', "/"));
    let toggled: Vec<String> = spellings
      .iter()
      .filter_map(|spelling| toggle_drive_case(spelling))
      .collect();
    spellings.extend(toggled);
  }

  let mut candidates = Vec::new();
  for spelling in &spellings {
    let json = json_escape(spelling);
    candidates.push(json_escape(&json));
    candidates.push(json);
    if spelling.contains('/') {
      candidates.push(replace_char(spelling, '/', "\\/"));
    }
    if !windows || !spelling.contains('\\') {
      candidates.push(file_url_path(spelling, windows));
    }
    candidates.push(spelling.clone());
  }
  candidates.sort_by_key(String::len);

  let mut patterns: Vec<Vec<u8>> = Vec::with_capacity(candidates.len());
  for candidate in candidates {
    let candidate = candidate.into_bytes();
    if !patterns.iter().any(|kept| contains(&candidate, kept)) {
      patterns.push(candidate);
    }
  }
  patterns
}

fn replace_char(value: &str, from: char, to: &str) -> String {
  let mut replaced = String::with_capacity(value.len());
  for c in value.chars() {
    if c == from {
      replaced.push_str(to);
    } else {
      replaced.push(c);
    }
  }
  replaced
}

fn is_windows_root(root: &str) -> bool {
  let bytes = root.as_bytes();
  root.starts_with("\\\\")
    || (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
}

/// The same path with its drive letter in the other case, or `None` without a drive letter.
fn toggle_drive_case(spelling: &str) -> Option<String> {
  let mut chars = spelling.chars();
  let drive = chars.next().filter(char::is_ascii_alphabetic)?;
  let rest = chars.as_str();
  if !rest.starts_with(':') {
    return None;
  }
  let toggled = if drive.is_ascii_uppercase() {
    drive.to_ascii_lowercase()
  } else {
    drive.to_ascii_uppercase()
  };
  Some(format!("{toggled}{rest}"))
}

/// The content of the string literal that `JSON.stringify(value)` produces.
fn json_escape(value: &str) -> String {
  let mut escaped = String::with_capacity(value.len());
  for c in value.chars() {
    match c {
      '"' => escaped.push_str("\\\""),
      '\\' => escaped.push_str("\\\\"),
      '\u{8}' => escaped.push_str("\\b"),
      '\u{c}' => escaped.push_str("\\f"),
      '\n' => escaped.push_str("\\n"),
      '\r' => escaped.push_str("\\r"),
      '\t' => escaped.push_str("\\t"),
      c if (c as u32) < 0x20 => {
        let _ = write!(escaped, "\\u{:04x}", c as u32);
      }
      c => escaped.push(c),
    }
  }
  escaped
}

fn file_url_path(slash: &str, windows: bool) -> String {
  let (mut url, rest, set) = if !windows {
    (String::new(), slash, SPECIAL_PATH_SEGMENT)
  } else if let Some(unc) = slash.strip_prefix("//") {
    let (host, rest) = unc.split_once('/').unwrap_or((unc, ""));
    let host = Host::parse(host).map_or_else(|_| host.to_string(), |host| host.to_string());
    (host, rest, PATH_SEGMENT)
  } else {
    let (drive, rest) = slash.split_at(2);
    (format!("/{drive}"), rest, PATH_SEGMENT)
  };
  for segment in rest.split('/').filter(|segment| !segment.is_empty()) {
    url.push('/');
    url.extend(utf8_percent_encode(segment, set));
  }
  if url.is_empty() {
    url.push('/');
  }
  url
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
  haystack
    .windows(needle.len())
    .any(|window| window == needle)
}
