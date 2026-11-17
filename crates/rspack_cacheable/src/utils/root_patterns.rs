use std::fmt::Write;

use CharEscape::{Brace, Raw, U, X};
use Hex::{Lower, Upper};
use RegexEscape::{HexHyphen, Hyphen, Standard, Syntax};
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use url::Host;

/// Root detection patterns and the conservative fallback for roots too short to detect.
#[derive(Debug)]
pub struct RootPatterns {
  /// Nonempty byte patterns to search for in serialized cache data.
  pub patterns: Vec<Vec<u8>>,
  /// Bind every entry when an alignment has no stable base64 character.
  pub bind_all: bool,
}

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

#[derive(Clone, Copy)]
enum Hex {
  Lower,
  Upper,
}

/// How a string literal spells one non-ASCII character.
#[derive(Clone, Copy)]
enum CharEscape {
  Raw,
  /// `\xHH`
  X(Hex),
  /// `\uHHHH`, as a surrogate pair above the BMP.
  U(Hex),
  /// `\u{H...}`
  Brace(Hex),
}

/// The spellings of a Latin-1, a BMP and an astral character that tools combine in one string.
/// Case belongs to each character class because SWC mixes it within a literal.
const ASCII_ESCAPES: [[CharEscape; 3]; 11] = [
  // SWC at ES5 or below with its default charset.
  [X(Lower), Raw, U(Upper)],
  // SWC with an ASCII charset, below and from ES2015.
  [X(Lower), U(Upper), U(Upper)],
  [X(Lower), U(Upper), Brace(Upper)],
  // Terser `ascii_only` and SWC regex sources, below and from ES2015.
  [X(Lower), U(Lower), U(Lower)],
  [X(Lower), U(Lower), Brace(Lower)],
  // esbuild and Babel, below and from ES2015.
  [X(Upper), U(Upper), U(Upper)],
  [X(Upper), U(Upper), Brace(Upper)],
  // Python `json.dumps`.
  [U(Lower), U(Lower), U(Lower)],
  // The TypeScript printer.
  [U(Upper), U(Upper), U(Upper)],
  // No surveyed tool emits these. A miss reuses a stale path and a false match only rebuilds.
  [U(Lower), U(Lower), Brace(Lower)],
  [U(Upper), U(Upper), Brace(Upper)],
];

/// What a regex escaper escapes besides the regex syntax characters.
#[derive(Clone, Copy)]
enum RegexEscape {
  /// Nothing else: the MDN snippet, lodash `escapeRegExp` and `escape-string-regexp` 1.
  Syntax,
  /// `-` as `\-`: a hand-written character class that lists `-`.
  Hyphen,
  /// `-` as `\x2d`: `escape-string-regexp` 5.
  HexHyphen,
  /// `RegExp.escape`: `/` as `\/`, and other punctuators and white space as hex escapes.
  Standard,
}

/// Byte patterns under which `root` can appear in serialized cache data.
///
/// Detection is best-effort. It covers the spellings that path-producing code commonly emits, and
/// it does not find arbitrary escapes, hashes, or other values computed from the root. For each
/// native spelling of the root, and for a Windows root also its forward-slash spelling and both
/// drive-letter cases, the patterns cover:
///
/// - the spelling itself, and the spelling with every `/` escaped as `\/`, as in JSON's escaped
///   solidus or a regex source;
/// - its `JSON.stringify` string content, once and nested twice;
/// - the path part of its `file://` URL, with a UNC host normalized the way
///   `url::Url::from_file_path` normalizes it;
/// - for a non-ASCII root, its `JSON.stringify` string content in each [`ASCII_ESCAPES`] spelling,
///   as is, with every `/` escaped as `\/`, and nested in JSON;
/// - the source of a regex that matches the spelling literally, as each [`RegexEscape`] builds it,
///   as is and as `JSON.stringify` string content, and for a non-ASCII root both of those in each
///   [`ASCII_ESCAPES`] spelling;
/// - the spelling and its JSON string content in base64, as in an inline source map.
///
/// On every host platform, a root that starts with a drive letter or `\\` is a Windows path.
/// Trailing separators do not matter. A pattern that contains another pattern is dropped, because
/// any match of it also matches the shorter one. Callers must bind every entry when `bind_all` is
/// set: a root such as `/` has no stable base64 character at one of the three byte alignments.
pub fn root_patterns(root: &str) -> RootPatterns {
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
  let mut bind_all = false;
  for spelling in &spellings {
    let json = json_escape(spelling);
    if !json.is_ascii() {
      for escapes in ASCII_ESCAPES {
        let escaped = ascii_escape(&json, escapes);
        candidates.push(json_escape(&escaped));
        let solidus = replace_char(&escaped, '/', "\\/");
        candidates.push(json_escape(&solidus));
        candidates.push(solidus);
        candidates.push(escaped);
      }
    }
    for value in [&json, spelling] {
      let (patterns, missing_alignment) = base64_spellings(value);
      candidates.extend(patterns);
      bind_all |= missing_alignment;
    }
    candidates.push(json_escape(&json));
    candidates.push(json);
    if spelling.contains('/') {
      let solidus = replace_char(spelling, '/', "\\/");
      candidates.push(json_escape(&solidus));
      candidates.push(solidus);
    }
    if !windows || !spelling.contains('\\') {
      candidates.push(file_url_path(spelling, windows));
    }
    candidates.push(spelling.clone());
    for source in regex_sources(spelling) {
      let json = json_escape(&source);
      if !json.is_ascii() {
        for escapes in ASCII_ESCAPES {
          candidates.push(ascii_escape(&json, escapes));
          candidates.push(ascii_escape(&source, escapes));
        }
      }
      candidates.push(json);
      candidates.push(source);
    }
  }
  candidates.sort_by_key(String::len);

  let mut patterns: Vec<Vec<u8>> = Vec::with_capacity(candidates.len());
  for candidate in candidates {
    let candidate = candidate.into_bytes();
    if !candidate.is_empty() && !patterns.iter().any(|kept| contains(&candidate, kept)) {
      patterns.push(candidate);
    }
  }
  RootPatterns { patterns, bind_all }
}

/// Base64 of `value` at each of the three byte alignments it can have inside a longer payload,
/// such as the JSON of an inline source map. Characters that also depend on neighbouring bytes
/// are left out. Keep even short fragments: a false match only rebuilds, but a miss reuses stale
/// output. Report any empty alignment so callers can bind every entry without an empty pattern.
fn base64_spellings(value: &str) -> (Vec<String>, bool) {
  let mut patterns = Vec::with_capacity(3);
  let mut bind_all = false;
  for padding in 0..3 {
    let mut bytes = vec![0; padding];
    bytes.extend_from_slice(value.as_bytes());
    bytes.extend_from_slice(&[0; 2]);
    let encoded = base64_simd::STANDARD.encode_to_string(&bytes);
    // Character i is stable iff 6*i >= 8*padding and 6*i + 6 <= 8*(padding + len).
    let start = [0, 2, 3][padding];
    let end = 8 * (padding + value.len()) / 6;
    if start >= end {
      bind_all = true;
    } else {
      patterns.push(encoded[start..end].to_string());
    }
  }
  (patterns, bind_all)
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

fn ascii_escape(value: &str, [latin1, bmp, astral]: [CharEscape; 3]) -> String {
  let mut escaped = String::with_capacity(value.len());
  for c in value.chars() {
    let char_escape = match c as u32 {
      0..=0x7f => Raw,
      0x80..=0xff => latin1,
      0x100..=0xffff => bmp,
      _ => astral,
    };
    let _ = match char_escape {
      Raw => write!(escaped, "{c}"),
      X(case) => write!(escaped, "\\x{}", hex(c as u32, 2, case)),
      U(case) => c
        .encode_utf16(&mut [0; 2])
        .iter()
        .try_for_each(|unit| write!(escaped, "\\u{}", hex(u32::from(*unit), 4, case))),
      Brace(case) => write!(escaped, "\\u{{{}}}", hex(c as u32, 0, case)),
    };
  }
  escaped
}

/// The sources of the regexes that match `spelling` literally, one per [`RegexEscape`]. A regex
/// source escapes `/` as `\/`, and the escaped string itself does not.
fn regex_sources(spelling: &str) -> Vec<String> {
  let mut sources = Vec::new();
  for style in [Syntax, Hyphen, HexHyphen] {
    let escaped = regex_escape(spelling, style);
    sources.push(replace_char(&escaped, '/', "\\/"));
    sources.push(escaped);
  }
  let standard = regex_escape(spelling, Standard);
  // `RegExp.escape` spells a letter or digit that starts its input as `\xHH`.
  if let Some(first) = spelling.chars().next().filter(char::is_ascii_alphanumeric) {
    sources.push(format!(
      "\\x{}{}",
      hex(first as u32, 2, Lower),
      &standard[1..]
    ));
  }
  sources.push(standard);
  sources
}

fn regex_escape(value: &str, style: RegexEscape) -> String {
  let mut escaped = String::with_capacity(value.len());
  for c in value.chars() {
    let _ = match (c, style) {
      ('\\' | '^' | '$' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|', _)
      | ('-', Hyphen)
      | ('/', Standard) => write!(escaped, "\\{c}"),
      ('-', HexHyphen) => write!(escaped, "\\x2d"),
      ('\t', Standard) => write!(escaped, "\\t"),
      ('\n', Standard) => write!(escaped, "\\n"),
      ('\u{b}', Standard) => write!(escaped, "\\v"),
      ('\u{c}', Standard) => write!(escaped, "\\f"),
      ('\r', Standard) => write!(escaped, "\\r"),
      // The other punctuators, and ECMAScript `WhiteSpace` and `LineTerminator`.
      (_, Standard)
        if ",-=<>#&!%:;@~'`\"".contains(c)
          || c == '\u{feff}'
          || (c.is_whitespace() && c != '\u{85}') =>
      {
        if c as u32 <= 0xff {
          write!(escaped, "\\x{}", hex(c as u32, 2, Lower))
        } else {
          c.encode_utf16(&mut [0; 2])
            .iter()
            .try_for_each(|unit| write!(escaped, "\\u{}", hex(u32::from(*unit), 4, Lower)))
        }
      }
      _ => write!(escaped, "{c}"),
    };
  }
  escaped
}

fn hex(value: u32, width: usize, case: Hex) -> String {
  match case {
    Lower => format!("{value:0width$x}"),
    Upper => format!("{value:0width$X}"),
  }
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
