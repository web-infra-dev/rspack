/*!
 * This Rust version of `is-glob` is based on the JavaScript library `is-glob`
 * is-glob <https://github.com/jonschlinkert/is-glob>
 *
 * Copyright (c) 2014-2017, Jon Schlinkert.
 * Released under the MIT License.
 */

use crate::is_extglob;

fn check(s: &str) -> bool {
  if s.starts_with('!') {
    return true;
  }

  let chars = s.chars().collect::<Vec<_>>();
  let mut index = 0;
  let mut pipe_index = -2;
  let mut close_square_index = -2;
  let mut close_curly_index = -2;
  let mut close_paren_index = -2;
  let mut back_slash_index = -2;

  while index < chars.len() {
    let current_char = chars[index];

    if current_char == '*' {
      return true;
    }

    if chars.get(index + 1) == Some(&'?') && [']', '.', '+', ')'].contains(&current_char) {
      return true;
    }

    if close_square_index != -1 && current_char == '[' && chars.get(index + 1) != Some(&']') {
      if close_square_index < index as i32 {
        close_square_index = chars
          .iter()
          .skip(index)
          .position(|c| *c == ']')
          .map(|i| (i + index) as i32)
          .unwrap_or(-1);
      }

      if close_square_index > index as i32 {
        if back_slash_index == -1 || back_slash_index > close_square_index {
          return true;
        }
        back_slash_index = chars
          .iter()
          .skip(index)
          .position(|c| *c == '\\')
          .map(|i| (i + index) as i32)
          .unwrap_or(-1);
        if back_slash_index == -1 || back_slash_index > close_square_index {
          return true;
        }
      }
    }

    if close_curly_index != -1 && current_char == '{' && chars.get(index + 1) != Some(&'}') {
      close_curly_index = chars
        .iter()
        .skip(index)
        .position(|c| *c == '}')
        .map(|i| (i + index) as i32)
        .unwrap_or(-1);
      if close_curly_index > index as i32 {
        back_slash_index = chars
          .iter()
          .skip(index)
          .position(|c| *c == '\\')
          .map(|i| (i + index) as i32)
          .unwrap_or(-1);
        if back_slash_index == -1 || back_slash_index > close_curly_index {
          return true;
        }
      }
    }

    if close_paren_index != -1
      && current_char == '('
      && chars.get(index + 1) == Some(&'?')
      && [':', '!', '='].contains(chars.get(index + 2).unwrap_or(&' '))
      && chars.get(index + 3) != Some(&')')
    {
      close_paren_index = chars
        .iter()
        .skip(index)
        .position(|c| *c == ')')
        .map(|i| (i + index) as i32)
        .unwrap_or(-1);
      if close_paren_index > index as i32 {
        back_slash_index = chars
          .iter()
          .skip(index)
          .position(|c| *c == '\\')
          .map(|i| (i + index) as i32)
          .unwrap_or(-1);
        if back_slash_index == -1 || back_slash_index > close_paren_index {
          return true;
        }
      }
    }

    if pipe_index != -1 && current_char == '(' && chars.get(index + 1) != Some(&'|') {
      if pipe_index < index as i32 {
        pipe_index = chars
          .iter()
          .skip(index)
          .position(|c| *c == '|')
          .map(|i| (i + index) as i32)
          .unwrap_or(-1);
      }
      if pipe_index != -1 && chars.get((pipe_index + 1) as usize) != Some(&')') {
        close_paren_index = chars
          .iter()
          .skip(pipe_index as usize)
          .position(|c| *c == ')')
          .map(|i| i as i32 + pipe_index)
          .unwrap_or(-1);
        if close_paren_index > pipe_index as i32 {
          back_slash_index = chars[pipe_index as usize..]
            .iter()
            .skip(pipe_index as usize)
            .position(|c| *c == '\\')
            .map(|i| i as i32 + pipe_index)
            .unwrap_or(-1);
          if back_slash_index == -1 || back_slash_index > close_paren_index {
            return true;
          }
        }
      }
    }

    if current_char == '\\' {
      if let Some(open) = chars.get(index + 1) {
        index += 2;
        let close = match *open {
          '{' => Some('}'),
          '(' => Some(')'),
          '[' => Some(']'),
          _ => None,
        };

        if let Some(close) = close {
          if let Some(n) = chars
            .iter()
            .skip(index)
            .position(|c| *c == close)
            .map(|i| i + index)
          {
            index += n + 1;
          }
        }

        if chars.get(index) == Some(&'!') {
          return true;
        }
      }
    } else {
      index += 1;
    }
  }

  return false;
}

pub fn is_glob(s: &str) -> bool {
  if s.is_empty() {
    return false;
  }

  if is_extglob::is_extglob(s) {
    return true;
  }

  check(s)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_glob_patterns() {
    assert!(!is_glob("@.(?abc)"), "invalid pattern");
    assert!(is_glob("*.js"));
    assert!(is_glob("!*.js"));
    assert!(is_glob("!foo"));
    assert!(is_glob("!foo.js"));
    assert!(is_glob("**/abc.js"));
    assert!(is_glob("abc/*.js"));
    assert!(is_glob("@.(?:abc)"));
    assert!(is_glob("@.(?!abc)"));
  }

  #[test]
  fn test_not_match_escaped_globs() {
    assert!(!is_glob("\\!\\*.js"));
    assert!(!is_glob("\\!foo"));
    assert!(!is_glob("\\!foo.js"));
    assert!(!is_glob("\\*(foo).js"));
    assert!(!is_glob("\\*.js"));
    assert!(!is_glob("\\*\\*/abc.js"));
    assert!(!is_glob("abc/\\*.js"));
  }

  #[test]
  fn test_not_glob_patterns() {
    assert!(!is_glob(""));
    assert!(!is_glob("~/abc"));
    assert!(!is_glob("~/(abc)"));
    assert!(!is_glob("+~(abc)"));
    assert!(!is_glob("."));
    assert!(!is_glob("@.(abc)"));
    assert!(!is_glob("aa"));
    assert!(!is_glob("who?"));
    assert!(!is_glob("why!?"));
    assert!(!is_glob("where???"));
    assert!(!is_glob("abc!/def/!ghi.js"));
    assert!(!is_glob("abc.js"));
    assert!(!is_glob("abc/def/!ghi.js"));
    assert!(!is_glob("abc/def/ghi.js"));
  }

  #[test]
  fn test_regex_capture_groups() {
    assert!(is_glob("abc/(?!foo).js"));
    assert!(is_glob("abc/(?:foo).js"));
    assert!(is_glob("abc/(?=foo).js"));
    assert!(is_glob("abc/(a|b).js"));
    assert!(is_glob("abc/(a|b|c).js"));
    assert!(
      is_glob("abc/(foo bar)/*.js"),
      "not a capture group but has a glob"
    );
  }

  #[test]
  fn test_not_regex_capture_groups() {
    assert!(!is_glob("abc/(?foo).js"), "invalid capture group");
    assert!(!is_glob("abc/(a b c).js"), "unlikely to be a capture group");
    assert!(!is_glob("abc/(ab).js"), "unlikely to be a capture group");
    assert!(!is_glob("abc/(abc).js"), "unlikely to be a capture group");
    assert!(
      !is_glob("abc/(foo bar).js"),
      "unlikely to be a capture group"
    );
  }

  #[test]
  fn test_imbalanced_capture_group() {
    assert!(!is_glob("abc/(?ab.js"));
    assert!(!is_glob("abc/(ab.js"));
    assert!(!is_glob("abc/(a|b.js"));
    assert!(!is_glob("abc/(a|b|c.js"));
  }

  #[test]
  fn test_escaped_capture_group() {
    assert!(!is_glob("abc/\\(a|b).js"));
    assert!(!is_glob("abc/\\(a|b|c).js"));
  }

  #[test]
  fn test_regex_character_classes() {
    assert!(is_glob("abc/[abc].js"));
    assert!(is_glob("abc/[^abc].js"));
    assert!(is_glob("abc/[1-3].js"));
  }

  #[test]
  fn test_imbalanced_character_classes() {
    assert!(!is_glob("abc/[abc.js"));
    assert!(!is_glob("abc/[^abc.js"));
    assert!(!is_glob("abc/[1-3.js"));
  }

  #[test]
  fn test_escaped_character_classes() {
    assert!(!is_glob("abc/\\[abc].js"));
    assert!(!is_glob("abc/\\[^abc].js"));
    assert!(!is_glob("abc/\\[1-3].js"));
  }

  #[test]
  fn test_brace_patterns() {
    assert!(is_glob("abc/{a,b}.js"));
    assert!(is_glob("abc/{a..z}.js"));
    assert!(is_glob("abc/{a..z..2}.js"));
  }

  #[test]
  fn test_balanced_braces() {
    assert!(!is_glob("abc/\\{a,b}.js"));
    assert!(!is_glob("abc/\\{a..z}.js"));
    assert!(!is_glob("abc/\\{a..z..2}.js"));
  }

  #[test]
  fn test_regex_patterns() {
    assert!(!is_glob("$(abc)"));
    assert!(!is_glob("&(abc)"));
    assert!(!is_glob("Who?.js"));
    assert!(!is_glob("? (abc)"));
    assert!(!is_glob("?.js"));
    assert!(!is_glob("abc/?.js"));

    assert!(is_glob("!&(abc)"));
    assert!(is_glob("!*.js"));
    assert!(is_glob("!foo"));
    assert!(is_glob("!foo.js"));
    assert!(is_glob("**/abc.js"));
    assert!(is_glob("*.js"));
    assert!(is_glob("*z(abc)"));
    assert!(is_glob("[1-10].js"));
    assert!(is_glob("[^abc].js"));
    assert!(is_glob("[a-j]*[^c]b/c"));
    assert!(is_glob("[abc].js"));
    assert!(is_glob("a/b/c/[a-z].js"));
    assert!(is_glob("abc/(aaa|bbb).js"));
    assert!(is_glob("abc/*.js"));
    assert!(is_glob("abc/{a,b}.js"));
    assert!(is_glob("abc/{a..z..2}.js"));
    assert!(is_glob("abc/{a..z}.js"));
  }

  #[test]
  fn test_escaped_regex_patterns() {
    assert!(!is_glob("\\?.js"));
    assert!(!is_glob("\\[1-10\\].js"));
    assert!(!is_glob("\\[^abc\\].js"));
    assert!(!is_glob("\\[a-j\\]\\*\\[^c\\]b/c"));
    assert!(!is_glob("\\[abc\\].js"));
    assert!(!is_glob("\\a/b/c/\\[a-z\\].js"));
    assert!(!is_glob("abc/\\(aaa|bbb).js"));
    assert!(!is_glob("abc/\\?.js"));
  }

  #[test]
  fn test_extglob_patterns() {
    assert!(is_glob("abc/!(a).js"));
    assert!(is_glob("abc/!(a|b).js"));
    assert!(is_glob("abc/(ab)*.js"));
    assert!(is_glob("abc/(a|b).js"));
    assert!(is_glob("abc/*(a).js"));
    assert!(is_glob("abc/*(a|b).js"));
    assert!(is_glob("abc/+(a).js"));
    assert!(is_glob("abc/+(a|b).js"));
    assert!(is_glob("abc/?(a).js"));
    assert!(is_glob("abc/?(a|b).js"));
    assert!(is_glob("abc/@(a).js"));
    assert!(is_glob("abc/@(a|b).js"));
  }

  #[test]
  fn test_escaped_extglob_patterns() {
    assert!(!is_glob("abc/\\*.js"));
    assert!(!is_glob("abc/\\*\\*.js"));
    assert!(!is_glob("abc/\\@(a).js"));
    assert!(!is_glob("abc/\\!(a).js"));
    assert!(!is_glob("abc/\\+(a).js"));
    assert!(!is_glob("abc/\\*(a).js"));
    assert!(!is_glob("abc/\\?(a).js"));
    assert!(
      is_glob("abc/\\@(a|b).js"),
      "matches since extglob is not escaped"
    );
    assert!(
      is_glob("abc/\\!(a|b).js"),
      "matches since extglob is not escaped"
    );
    assert!(
      is_glob("abc/\\+(a|b).js"),
      "matches since extglob is not escaped"
    );
    assert!(
      is_glob("abc/\\*(a|b).js"),
      "matches since extglob is not escaped"
    );
    assert!(
      is_glob("abc/\\?(a|b).js"),
      "matches since extglob is not escaped"
    );
    assert!(
      is_glob("abc/\\@(a\\|b).js"),
      "matches since extglob is not escaped"
    );
    assert!(
      is_glob("abc/\\!(a\\|b).js"),
      "matches since extglob is not escaped"
    );
    assert!(
      is_glob("abc/\\+(a\\|b).js"),
      "matches since extglob is not escaped"
    );
    assert!(
      is_glob("abc/\\*(a\\|b).js"),
      "matches since extglob is not escaped"
    );
    assert!(
      is_glob("abc/\\?(a\\|b).js"),
      "matches since extglob is not escaped"
    );
  }

  #[test]
  fn test_non_extglob_parens() {
    assert!(!is_glob("C:/Program Files (x86)/"));
  }

  #[test]
  fn test_glob_chars_invalid_path() {
    assert!(is_glob("abc/[*].js"));
    assert!(is_glob("abc/*.js"));
  }

  #[test]
  fn test_valid_non_glob_path() {
    assert!(!is_glob("abc/?.js"));
    assert!(!is_glob("abc/!.js"));
    assert!(!is_glob("abc/@.js"));
    assert!(!is_glob("abc/+.js"));
  }

  #[test]
  fn test_is_glob_with_extglob() {
    assert!(is_glob("?(abc)"));
    assert!(is_glob("@(abc)"));
    assert!(is_glob("!(abc)"));
    assert!(is_glob("*(abc)"));
    assert!(is_glob("+(abc)"));
    assert!(is_glob("xyz/?(abc)/xyz"));
    assert!(is_glob("xyz/@(abc)/xyz"));
    assert!(is_glob("xyz/!(abc)/xyz"));
    assert!(is_glob("xyz/*(abc)/xyz"));
    assert!(is_glob("xyz/+(abc)/xyz"));
    assert!(is_glob("?(abc|xyz)/xyz"));
    assert!(is_glob("@(abc|xyz)"));
    assert!(is_glob("!(abc|xyz)"));
    assert!(is_glob("*(abc|xyz)"));
    assert!(is_glob("+(abc|xyz)"));
  }

  #[test]
  fn test_not_match_escaped_extglobs() {
    assert!(!is_glob("\\?(abc)"));
    assert!(!is_glob("\\@(abc)"));
    assert!(!is_glob("\\!(abc)"));
    assert!(!is_glob("\\*(abc)"));
    assert!(!is_glob("\\+(abc)"));
    assert!(!is_glob("xyz/\\?(abc)/xyz"));
    assert!(!is_glob("xyz/\\@(abc)/xyz"));
    assert!(!is_glob("xyz/\\!(abc)/xyz"));
    assert!(!is_glob("xyz/\\*(abc)/xyz"));
    assert!(!is_glob("xyz/\\+(abc)/xyz"));
  }

  #[test]
  fn test_same_pattern_escaped_and_unescaped_glob() {
    assert!(is_glob("\\?(abc)/?(abc)"));
    assert!(is_glob("\\@(abc)/@(abc)"));
    assert!(is_glob("\\!(abc)/!(abc)"));
    assert!(is_glob("\\*(abc)/*(abc)"));
    assert!(is_glob("\\+(abc)/+(abc)"));
    assert!(is_glob("xyz/\\?(abc)/xyz/xyz/?(abc)/xyz"));
    assert!(is_glob("xyz/\\@(abc)/xyz/xyz/@(abc)/xyz"));
    assert!(is_glob("xyz/\\!(abc)/xyz/xyz/!(abc)/xyz"));
    assert!(is_glob("xyz/\\*(abc)/xyz/xyz/*(abc)/xyz"));
    assert!(is_glob("xyz/\\+(abc)/xyz/xyz/+(abc)/xyz"));
    assert!(is_glob("\\?(abc|xyz)/xyz/?(abc|xyz)/xyz"));
    assert!(is_glob("\\@(abc|xyz)/@(abc|xyz)"));
    assert!(is_glob("\\!(abc|xyz)/!(abc|xyz)"));
    assert!(is_glob("\\*(abc|xyz)/*(abc|xyz)"));
    assert!(is_glob("\\+(abc|xyz)/+(abc|xyz)"));
  }
}
