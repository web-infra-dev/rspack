/*!
 * This Rust version of `is-extglob` is based on the JavaScript library `is-extglob`
 * is-extglob <https://github.com/jonschlinkert/is-extglob>
 *
 * Copyright (c) 2014-2016, Jon Schlinkert.
 * Licensed under the MIT License.
 */

use regex::Regex;

// This Rust version of `is-extglob` is based on the JavaScript library `is-extglob`
pub fn is_extglob(s: &str) -> bool {
  if s.is_empty() {
    return false;
  }

  let re = Regex::new(r"(\\).|([@?!+*]\(.*\))").unwrap();

  let mut input = s;
  while let Some(captures) = re.captures(input) {
    if captures.get(2).is_some() {
      return true;
    }
    if let Some(matched) = captures.get(0) {
      input = &input[matched.end()..];
    }
  }

  false
}

#[cfg(test)]
mod tests {
  use super::is_extglob;

  #[test]
  fn test_is_extglob() {
    assert!(is_extglob("?(abc)"));
    assert!(is_extglob("@(abc)"));
    assert!(is_extglob("!(abc)"));
    assert!(is_extglob("*(abc)"));
    assert!(is_extglob("+(abc)"));
    assert!(is_extglob("xyz/?(abc)/xyz"));
    assert!(is_extglob("xyz/@(abc)/xyz"));
    assert!(is_extglob("xyz/!(abc)/xyz"));
    assert!(is_extglob("xyz/*(abc)/xyz"));
    assert!(is_extglob("xyz/+(abc)/xyz"));
    assert!(is_extglob("?(abc|xyz)/xyz"));
    assert!(is_extglob("@(abc|xyz)"));
    assert!(is_extglob("!(abc|xyz)"));
    assert!(is_extglob("*(abc|xyz)"));
    assert!(is_extglob("+(abc|xyz)"));
  }

  #[test]
  fn test_not_match_escaped_extglobs() {
    assert!(!is_extglob("?(abc/xyz"));
    assert!(!is_extglob("@(abc"));
    assert!(!is_extglob("!(abc"));
    assert!(!is_extglob("*(abc"));
    assert!(!is_extglob("+(abc"));
    assert!(!is_extglob("(a|b"));
    assert!(!is_extglob("\\?(abc)"));
    assert!(!is_extglob("\\@(abc)"));
    assert!(!is_extglob("\\!(abc)"));
    assert!(!is_extglob("\\*(abc)"));
    assert!(!is_extglob("\\+(abc)"));
    assert!(!is_extglob("xyz/\\?(abc)/xyz"));
    assert!(!is_extglob("xyz/\\@(abc)/xyz"));
    assert!(!is_extglob("xyz/\\!(abc)/xyz"));
    assert!(!is_extglob("xyz/\\*(abc)/xyz"));
    assert!(!is_extglob("xyz/\\+(abc)/xyz"));
    assert!(!is_extglob("\\?(abc|xyz)/xyz"));
    assert!(!is_extglob("\\@(abc|xyz)"));
    assert!(!is_extglob("\\!(abc|xyz)"));
    assert!(!is_extglob("\\*(abc|xyz)"));
    assert!(!is_extglob("\\+(abc|xyz)"));
    assert!(!is_extglob("?\\(abc)"));
    assert!(!is_extglob("@\\(abc)"));
    assert!(!is_extglob("!\\(abc)"));
    assert!(!is_extglob("*\\(abc)"));
    assert!(!is_extglob("+\\(abc)"));
    assert!(!is_extglob("xyz/?\\(abc)/xyz"));
    assert!(!is_extglob("xyz/@\\(abc)/xyz"));
    assert!(!is_extglob("xyz/!\\(abc)/xyz"));
    assert!(!is_extglob("xyz/*\\(abc)/xyz"));
    assert!(!is_extglob("xyz/+\\(abc)/xyz"));
    assert!(!is_extglob("?\\(abc|xyz)/xyz"));
    assert!(!is_extglob("@\\(abc|xyz)"));
    assert!(!is_extglob("!\\(abc|xyz)"));
    assert!(!is_extglob("*\\(abc|xyz)"));
    assert!(!is_extglob("+\\(abc|xyz)"));
  }

  #[test]
  fn test_extglob_in_same_pattern_with_escaped_extglob() {
    assert!(is_extglob("\\?(abc)/?(abc)"));
    assert!(is_extglob("\\@(abc)/@(abc)"));
    assert!(is_extglob("\\!(abc)/!(abc)"));
    assert!(is_extglob("\\*(abc)/*(abc)"));
    assert!(is_extglob("\\+(abc)/+(abc)"));
    assert!(is_extglob("xyz/\\?(abc)/xyz/xyz/?(abc)/xyz"));
    assert!(is_extglob("xyz/\\@(abc)/xyz/xyz/@(abc)/xyz"));
    assert!(is_extglob("xyz/\\!(abc)/xyz/xyz/!(abc)/xyz"));
    assert!(is_extglob("xyz/\\*(abc)/xyz/xyz/*(abc)/xyz"));
    assert!(is_extglob("xyz/\\+(abc)/xyz/xyz/+(abc)/xyz"));
    assert!(is_extglob("\\?(abc|xyz)/xyz/?(abc|xyz)/xyz"));
    assert!(is_extglob("\\@(abc|xyz)/@(abc|xyz)"));
    assert!(is_extglob("\\!(abc|xyz)/!(abc|xyz)"));
    assert!(is_extglob("\\*(abc|xyz)/*(abc|xyz)"));
    assert!(is_extglob("\\+(abc|xyz)/+(abc|xyz)"));
  }

  #[test]
  fn test_no_extglob() {
    assert!(!is_extglob(""));
    assert!(!is_extglob("? (abc)"));
    assert!(!is_extglob("@.(abc)"));
    assert!(!is_extglob("!&(abc)"));
    assert!(!is_extglob("*z(abc)"));
    assert!(!is_extglob("+~(abc)"));
    assert!(!is_extglob("abc/{a,b}.js"));
    assert!(!is_extglob("abc/{a..z}.js"));
    assert!(!is_extglob("abc/{a..z..2}.js"));
    assert!(!is_extglob("abc/(aaa|bbb).js"));
    assert!(!is_extglob("abc/?.js"));
    assert!(!is_extglob("?.js"));
    assert!(!is_extglob("[abc].js"));
    assert!(!is_extglob("[^abc].js"));
    assert!(!is_extglob("a/b/c/[a-z].js"));
    assert!(!is_extglob("[a-j]*[^c]b/c"));
    assert!(!is_extglob("."));
    assert!(!is_extglob("aa"));
    assert!(!is_extglob("abc.js"));
    assert!(!is_extglob("abc/def/ghi.js"));
  }
}
