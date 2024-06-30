use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scheme {
  None,
  Data,
  File,
  Http,
  Custom(String),
}

impl Scheme {
  pub fn is_file(&self) -> bool {
    matches!(self, Self::File)
  }

  pub fn is_data(&self) -> bool {
    matches!(self, Self::Data)
  }

  pub fn is_none(&self) -> bool {
    matches!(self, Self::None)
  }

  pub fn is_some(&self) -> bool {
    !self.is_none()
  }

  pub fn is_http(&self) -> bool {
    matches!(self, Self::Http)
  }
}

impl From<&str> for Scheme {
  fn from(value: &str) -> Self {
    match value {
      "" => Self::None,
      // To avoid conflict with builtin loader protocol
      "builtin" => Self::None,
      "data" => Self::Data,
      "file" => Self::File,
      "http" => Self::Http,
      v => Self::Custom(v.to_string()),
    }
  }
}

impl fmt::Display for Scheme {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::None => "",
        Self::Data => "data",
        Self::File => "file",
        Self::Http => "http",
        Self::Custom(v) => v,
      }
    )
  }
}

const BACK_SLASH: char = '\\';
const SLASH: char = '/';
const A_LOWER_CASE: char = 'a';
const Z_LOWER_CASE: char = 'z';
const A_UPPER_CASE: char = 'A';
const Z_UPPER_CASE: char = 'Z';
const ZERO: char = '0';
const NINE: char = '9';
const PLUS: char = '+';
const HYPHEN: char = '-';
const COLON: char = ':';
const HASH: char = '#';
const QUERY: char = '?';

pub fn get_scheme(specifier: &str) -> Scheme {
  let mut chars = specifier.chars().enumerate().peekable();

  // First char maybe only a letter
  let start = chars.next();
  if start.is_none()
    || matches!(start, Some((_, c)) if (c < A_LOWER_CASE || c > Z_LOWER_CASE) && (c < A_UPPER_CASE || c > Z_UPPER_CASE))
  {
    return Scheme::None;
  }

  let mut next = chars.next();
  while let Some((_, ch)) = next
    && ((ch >= A_LOWER_CASE && ch <= Z_LOWER_CASE)
      || (ch >= A_UPPER_CASE && ch <= Z_UPPER_CASE)
      || (ch >= ZERO && ch <= NINE)
      || ch == PLUS
      || ch == HYPHEN)
  {
    if chars.peek().is_none() {
      return Scheme::None;
    }
    next = chars.next();
  }

  // Scheme must end with colon
  let maybe_colon = next;
  if maybe_colon.is_none() || matches!(maybe_colon, Some((_, c)) if c != COLON) {
    return Scheme::None;
  }

  // Check for Windows absolute path
  // https://url.spec.whatwg.org/#url-miscellaneous
  let (i, _) = maybe_colon.expect("should not be None");
  if i == 1 {
    let next_ch = chars.next();
    if next_ch.is_none()
      || matches!(next_ch, Some((_, ch)) if ch == BACK_SLASH || ch == SLASH || ch == HASH || ch == QUERY)
    {
      return Scheme::None;
    }
  }

  Scheme::from(specifier[..i].to_ascii_lowercase().as_str())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn none_for_windows_path() {
    assert_eq!(get_scheme("D:\\a\\rspack\\index.js"), Scheme::None);
  }

  #[test]
  fn data_for_data_uri() {
    assert_eq!(get_scheme("data:text/javascript"), Scheme::Data);
  }

  #[test]
  fn http_for_http_url() {
    assert_eq!(get_scheme("http://localhost"), Scheme::Http);
    assert!(Scheme::Http.is_http());
  }

  #[test]
  fn file_for_file_url() {
    assert_eq!(get_scheme("file:/a.js"), Scheme::File);
  }

  #[test]
  fn custom_for_custom_url() {
    assert_eq!(
      get_scheme("native:/Users/a.js"),
      Scheme::Custom("native".to_owned())
    );
  }
}
