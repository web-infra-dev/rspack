use std::borrow::Cow;

use rspack_paths::Utf8Path;
use rspack_util::identifier::push_absolute_to_request;
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

  for (index, byte) in request.bytes().enumerate() {
    if byte == b'!' {
      push_absolute_to_request(context, &request[last..index], &mut result);
      result.push('!');
      last = index + 1;
    }
  }

  push_absolute_to_request(context, &request[last..], &mut result);
  result
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
