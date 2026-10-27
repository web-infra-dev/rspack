use std::borrow::Cow;

use memchr::memchr_iter;
use rspack_paths::Utf8Path;
use rspack_sources::SourceMap;
use rspack_util::identifier::{make_paths_relative, push_absolute_to_request};
use swc_core::ecma::utils::is_valid_prop_ident;

use crate::BoxLoader;

pub fn to_module_export_name(name: &str) -> String {
  if is_valid_prop_ident(name) {
    name.into()
  } else {
    rspack_util::json_stringify_str(name)
  }
}

pub fn contextify(context: impl AsRef<Utf8Path>, request: &str) -> String {
  let context = context.as_ref().as_str();
  let mut result = String::with_capacity(request.len());
  let mut last = 0;

  for index in memchr_iter(b'!', request.as_bytes()) {
    push_absolute_to_request(context, &request[last..index], &mut result);
    result.push('!');
    last = index + 1;
  }

  push_absolute_to_request(context, &request[last..], &mut result);
  result
}

/// Converts a source path to a context-relative `webpack://` URL, preserving existing URLs.
///
/// `file://` URLs, and `webpack://` URLs that hold an absolute path on the same root as
/// `context`, are converted too, so build-local paths don't leak into module hashes.
pub fn contextify_source_url(context: &str, source: &str) -> String {
  if let Some(path) = source.strip_prefix("file://").and_then(file_url_path) {
    return to_context_relative_url(context, &path);
  }
  if let Some(path) = source
    .strip_prefix("webpack://")
    .and_then(|path| absolute_path_on_context_root(context, path))
  {
    return to_context_relative_url(context, path);
  }
  if source.starts_with("webpack://")
    || source.starts_with("rspack://")
    || source.starts_with("data:")
    || source.starts_with("http:")
    || source.starts_with("https:")
  {
    return source.to_string();
  }
  to_context_relative_url(context, source)
}

fn to_context_relative_url(context: &str, path: &str) -> String {
  let mut result = String::with_capacity("webpack://".len() + context.len() + path.len());
  result.push_str("webpack://");
  result.push_str(&make_paths_relative(context, path));
  result
}

/// Returns the decoded path of a `file://` URL with an empty host (`file:///abs/path`).
/// URLs naming a host are left alone.
fn file_url_path(url_path: &str) -> Option<String> {
  if !url_path.starts_with('/') {
    return None;
  }
  let path = urlencoding::decode(url_path).ok()?;
  Some(strip_slash_before_drive(&path).to_string())
}

/// `webpack://` URLs usually hold `[namespace]/[resource-path]`, which is already relative.
/// Only absolute paths whose first segment matches `context`'s are rewritten, so virtual
/// modules such as `webpack:///webpack/bootstrap` keep their URL.
fn absolute_path_on_context_root<'a>(context: &str, path: &'a str) -> Option<&'a str> {
  let path = strip_slash_before_drive(path);
  if !path.starts_with('/') && !is_windows_drive_path(path) {
    return None;
  }
  let context_root = root_segment(context)?;
  (root_segment(path) == Some(context_root)).then_some(path)
}

fn root_segment(path: &str) -> Option<&str> {
  path
    .trim_start_matches('/')
    .split(['/', '\\'])
    .next()
    .filter(|segment| !segment.is_empty())
}

/// URLs write Windows paths as `/C:/path`; drop the leading slash so they read as absolute.
fn strip_slash_before_drive(path: &str) -> &str {
  match path.strip_prefix('/') {
    Some(rest) if is_windows_drive_path(rest) => rest,
    _ => path,
  }
}

fn is_windows_drive_path(path: &str) -> bool {
  let bytes = path.as_bytes();
  bytes.len() >= 3
    && bytes[0].is_ascii_alphabetic()
    && bytes[1] == b':'
    && matches!(bytes[2], b'/' | b'\\')
}

/// Matches absolute sources that override `sourceRoot` in `rspack_sources::helpers::get_source`.
fn is_absolute_source(source: &str) -> bool {
  if source.starts_with('/') {
    return true;
  }
  let Some((scheme, _)) = source.split_once(':') else {
    return false;
  };
  let mut chars = scheme.chars();
  matches!(chars.next(), Some(first) if first.is_ascii_alphabetic())
    && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}

/// Removes context-dependent paths from a loader-provided source map.
pub fn contextify_source_map(context: &str, source_map: &mut SourceMap<'static>) {
  let source_root = source_map
    .source_root()
    .filter(|source_root| !source_root.is_empty())
    .map(str::to_string);
  let sources: Vec<String> = source_map
    .sources()
    .iter()
    .map(|source| {
      let source = match &source_root {
        Some(source_root) if !is_absolute_source(source) => {
          if source_root.ends_with('/') {
            format!("{source_root}{source}")
          } else {
            format!("{source_root}/{source}")
          }
        }
        _ => source.to_string(),
      };
      contextify_source_url(context, &source)
    })
    .collect();
  source_map.set_file(Some(Cow::Borrowed("x")));
  source_map.set_source_root(None);
  source_map.set_sources(sources);
}

#[inline]
fn is_ident_first_safe(b: u8) -> bool {
  matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'$' | b'_')
}

#[inline]
fn is_ident_safe(b: u8) -> bool {
  matches!(b, b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'$')
}

#[inline]
pub fn to_identifier(v: &str) -> Cow<'_, str> {
  let mut buf = if v
    .as_bytes()
    .first()
    .is_none_or(|&b| is_ident_first_safe(b) || !is_ident_safe(b))
  {
    String::new()
  } else {
    "_".into()
  };

  if escape_identifier_impl(v, &mut buf) {
    if buf.is_empty() {
      Cow::Borrowed(v)
    } else {
      buf.push_str(v);
      Cow::Owned(buf)
    }
  } else {
    Cow::Owned(buf)
  }
}

pub fn to_identifier_with_escaped(v: String) -> String {
  if v.is_empty() {
    return v;
  }

  let bytes = v.as_bytes();
  // Fast path: the input is already a valid JS identifier — skip the full
  // escape (see #10760). `_` is a valid continuation char even though
  // `is_ident_safe` excludes it (it's the replacement sentinel for the escape
  // impl), so handle it inline here.
  if is_ident_first_safe(bytes[0]) && bytes.iter().all(|&b| is_ident_safe(b) || b == b'_') {
    return v;
  }

  // Defensive path: invalid characters anywhere in the input (e.g. JSON keys
  // like "!top" or "with space") need the full escape so we never emit bare
  // invalid characters into a JS identifier position.
  to_identifier(&v).into_owned()
}

pub fn escape_identifier(v: &str) -> Cow<'_, str> {
  let mut buf = String::new();
  if escape_identifier_impl(v, &mut buf) {
    Cow::Borrowed(v)
  } else {
    Cow::Owned(buf)
  }
}

fn escape_identifier_impl(v: &str, out: &mut String) -> bool {
  let vstr = v;
  let v = v.as_bytes();

  // hot path
  let Some(first_invalid) = v.iter().position(|&b| !is_ident_safe(b)) else {
    return true;
  };

  // # Safety
  //
  // `first_invalid` is either an ASCII byte offset or the first byte of a
  // non-ASCII character, so slicing before it stays on a UTF-8 boundary.
  out.push_str(&vstr[..first_invalid]);
  out.push('_');

  let start = first_invalid + 1;
  let mut pos = start;
  let mut is_safe = false;
  for i in start..v.len() {
    if is_ident_safe(v[i]) {
      if !is_safe {
        pos = i;
        is_safe = true;
      }
    } else {
      if is_safe {
        // # Safety
        //
        // `pos` and `i` are ASCII byte offsets because they are only assigned
        // while `is_ident_safe` is true.
        out.push_str(&vstr[pos..i]);
        out.push('_');
        is_safe = false;
      }
    }
  }

  if is_safe {
    // # Safety
    //
    // `pos` is an ASCII byte offset because it is only assigned while
    // `is_ident_safe` is true.
    let s = &vstr[pos..];
    out.push_str(s);
  }

  false
}

pub fn stringify_loaders_and_resource<'a>(
  loaders: &'a [BoxLoader],
  resource: &'a str,
) -> Cow<'a, str> {
  if !loaders.is_empty() {
    let mut s = String::new();
    for loader in loaders {
      let identifier = loader.identifier();
      if let Some((_type, ident)) = identifier.split_once('|') {
        s.push_str(ident);
      } else {
        s.push_str(identifier.as_str());
      }
      s.push('!');
    }
    s.push_str(resource);
    Cow::Owned(s)
  } else {
    Cow::Borrowed(resource)
  }
}

#[test]
fn test_to_identifier() {
  assert_eq!(to_identifier("ident0"), "ident0");
  assert_eq!(to_identifier("0ident"), "_0ident");
  assert_eq!(to_identifier("/ident"), "_ident");
  assert_eq!(
    to_identifier("ident0_stable/src/core/iter//range.rs"),
    "ident0_stable_src_core_iter_range_rs"
  );
  assert_eq!(
    to_identifier("ident0_stable/src/core/iter//range.rs?"),
    "ident0_stable_src_core_iter_range_rs_"
  );
}

#[test]
fn test_to_identifier_with_escaped() {
  // Already-valid identifiers pass through unchanged.
  assert_eq!(to_identifier_with_escaped("ident0".into()), "ident0");
  assert_eq!(to_identifier_with_escaped("_top".into()), "_top");
  assert_eq!(to_identifier_with_escaped("$foo".into()), "$foo");

  // First-char-only fixups still work.
  assert_eq!(to_identifier_with_escaped("0ident".into()), "_0ident");

  // Invalid characters anywhere in the input are escaped — regression coverage
  // for JSON keys like "!top" / "with space" leaking into JS identifier positions.
  assert_eq!(to_identifier_with_escaped("!top".into()), "_top");
  assert_eq!(to_identifier_with_escaped("_!top".into()), "_top");
  assert_eq!(
    to_identifier_with_escaped("with space".into()),
    "with_space"
  );
  assert_eq!(to_identifier_with_escaped("a.b".into()), "a_b");
}

#[test]
fn test_contextify_preserves_loader_segments_and_queries() {
  assert_eq!(
    contextify(
      "/workspace/app",
      "/workspace/app/loaders/a.js!/workspace/app/src/index.js?foo=1"
    ),
    "./loaders/a.js!./src/index.js?foo=1"
  );
}

#[test]
fn test_contextify_preserves_empty_segments_and_regex_segments() {
  assert_eq!(
    contextify("/workspace/app", "!!/regexp/!/workspace/app/src/index.js"),
    "!!/regexp/!./src/index.js"
  );
}
