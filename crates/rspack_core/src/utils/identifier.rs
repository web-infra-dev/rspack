use std::borrow::Cow;

use rspack_paths::Utf8Path;
use rspack_util::identifier::absolute_to_request;

use crate::BoxLoader;

pub fn contextify(context: impl AsRef<Utf8Path>, request: &str) -> String {
  let context = context.as_ref();
  request
    .split('!')
    .map(|r| absolute_to_request(context.as_str(), r))
    .collect::<Vec<Cow<str>>>()
    .join("!")
}

fn is_ident_first_safe(b: u8) -> bool {
  // TODO const
  fn tablef() -> u128 {
    (b'A'..=b'Z')
      .chain(b'a'..=b'z')
      .chain(Some(b'$'))
      .chain(Some(b'_'))
      .fold(0, |mut sum, next| {
        sum |= 1 << next;
        sum
      })
  }

  #[allow(clippy::unreadable_literal)]
  let table: u128 = 10633823849912963253511222563025453056;
  debug_assert_eq!(table, tablef());
  b < 128 && (table & (1 << b)) != 0
}

fn is_ident_safe(b: u8) -> bool {
  // TODO const
  fn tablef() -> u128 {
    (b'0'..=b'9')
      .chain(b'A'..=b'Z')
      .chain(b'a'..=b'z')
      .chain(Some(b'$'))
      .fold(0, |mut sum, next| {
        sum |= 1 << next;
        sum
      })
  }

  #[allow(clippy::unreadable_literal)]
  let table: u128 = 10633823810298881996667002667428478976;
  debug_assert_eq!(table, tablef());
  b < 128 && (table & (1 << b)) != 0
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

  if let Some(first_char) = v.chars().next() {
    if first_char.is_ascii_alphabetic() || first_char == '$' || first_char == '_' {
      return v;
    }
    format!("_{v}")
  } else {
    v
  }
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
  let mut pos = 0;
  let mut is_safe = true;

  // hot path
  if v.iter().all(|&b| is_ident_safe(b)) {
    return true;
  }

  for i in 0..v.len() {
    match (is_ident_safe(v[i]), is_safe) {
      (true, true) => (),
      (false, true) => {
        // # Safety
        //
        // always ascii
        let s = &vstr[pos..i];
        out.push_str(s);
        out.push('_');
        pos = i + 1;
        is_safe = false;
      }
      (false, false) => pos = i + 1,
      (true, false) => is_safe = true,
    }
  }

  if pos != 0 {
    // # Safety
    //
    // always ascii
    let s = &vstr[pos..];
    out.push_str(s);
  }

  pos == 0
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
