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

fn replace_identifier_name_prefix(v: &str) -> Cow<str> {
  if v.is_empty() {
    return Cow::Borrowed(v);
  }
  
  let first_char = v.chars().next().unwrap();
  if first_char.is_ascii_alphabetic() || first_char == '$' || first_char == '_' {
    Cow::Borrowed(v)
  } else {
    Cow::Owned(format!("_{}", v))
  }
}

fn replace_non_identifier_chars(v: &str) -> Cow<str> {
  let mut result = String::new();
  let mut needs_replacement = false;
  
  for ch in v.chars() {
    if ch.is_ascii_alphanumeric() || ch == '$' {
      if needs_replacement {
        result.push(ch);
      }
    } else {
      if !needs_replacement {
        needs_replacement = true;
        result.reserve(v.len());
        // Copy the part we've checked so far
        for prev_ch in v.chars().take_while(|&c| c != ch) {
          result.push(prev_ch);
        }
      }
      result.push('_');
    }
  }
  
  if needs_replacement {
    Cow::Owned(result)
  } else {
    Cow::Borrowed(v)
  }
}

#[inline]
pub fn to_identifier(v: &str) -> Cow<'_, str> {
  // Avoid any unnecessary cost
  match replace_identifier_name_prefix(v) {
    Cow::Borrowed(_) => escape_identifier(v),
    Cow::Owned(id) => match escape_identifier(&id) {
      Cow::Borrowed(_unchanged) => Cow::Owned(id),
      Cow::Owned(id) => Cow::Owned(id),
    },
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
  replace_non_identifier_chars(v)
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
