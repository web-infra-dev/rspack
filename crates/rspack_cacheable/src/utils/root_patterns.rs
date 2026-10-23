use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

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
/// Covers the native spelling, the forward-slash spelling of a Windows root, the JSON-escaped
/// spellings, and the path part of the root's `file://` URL. The input is read as a Windows path
/// when it starts with a drive letter or `\\`, on every host platform. Trailing separators are
/// ignored. A pattern that contains another pattern is dropped, since any match of it is also a
/// match of the shorter one.
pub fn root_patterns(root: &str) -> Vec<Vec<u8>> {
  let windows = is_windows_root(root);
  let trimmed = root.trim_end_matches(|c| c == '/' || (windows && c == '\\'));
  let root = if trimmed.is_empty() { root } else { trimmed };
  let slash = if windows {
    root.replace('\\', "/")
  } else {
    root.to_string()
  };

  let mut candidates = vec![
    root.to_string(),
    json_escape(root),
    json_escape(&slash),
    file_url_path(&slash, windows),
    slash,
  ];
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

fn is_windows_root(root: &str) -> bool {
  let bytes = root.as_bytes();
  root.starts_with("\\\\")
    || (bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':')
}

fn json_escape(value: &str) -> String {
  value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn file_url_path(slash: &str, windows: bool) -> String {
  let (mut url, rest, set) = if !windows {
    (String::new(), slash, SPECIAL_PATH_SEGMENT)
  } else if let Some(unc) = slash.strip_prefix("//") {
    let (host, rest) = unc.split_once('/').unwrap_or((unc, ""));
    (host.to_string(), rest, PATH_SEGMENT)
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
