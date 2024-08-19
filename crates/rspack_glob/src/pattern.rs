use std::{borrow::Borrow, path::Path};

use regex::Regex;

use crate::is_glob;

pub fn is_negative_pattern(pattern: &str) -> bool {
  let mut chars = pattern.chars();
  match chars.next() {
    Some('!') => (),
    _ => return false,
  }
  match chars.next() {
    Some('(') => false,
    _ => return true,
  }
}

pub fn is_positive_pattern(pattern: &str) -> bool {
  !is_negative_pattern(pattern)
}

pub fn get_negative_patterns(patterns: &[String]) -> Vec<String> {
  patterns
    .iter()
    .filter(|&pattern| is_negative_pattern(pattern))
    .cloned()
    .collect()
}

pub fn get_positive_patterns(patterns: &[String]) -> Vec<String> {
  patterns
    .iter()
    .filter(|&pattern| is_positive_pattern(pattern))
    .cloned()
    .collect()
}

// Returns patterns to be expanded relative to (outside) the current directory.
//
// ['../*', './../*']
// get_patterns_outside_current_directory(vec!['./*', '*', 'a/*', '../*', './../*'])
pub fn get_patterns_outside_current_directory(patterns: &[String]) -> Vec<String> {
  patterns
    .iter()
    .filter(|&pattern| is_pattern_related_to_parent_directory(pattern))
    .cloned()
    .collect()
}

pub fn is_pattern_related_to_parent_directory(pattern: &String) -> bool {
  pattern.starts_with("..") || pattern.starts_with("./..")
}

// https://github.com/gulpjs/glob-parent/blob/main/index.js
pub fn get_base_directory(pattern: &str) -> String {
  let escaped_re = Regex::new(r#"\\([!*?|\[\](){}])"#).unwrap();

  let mut result = pattern.to_string();

  // special case for strings ending in enclosure containing path separator
  if is_enclosure(&result) {
    result.push('/');
  }

  // preserves full path in case of trailing path separator
  result.push('a');

  // remove path parts that are globby
  result = dirname(&result);
  while is_globby(&result) {
    result = dirname(&result);
  }

  // remove escape chars and return result
  escaped_re.replace_all(&result, "$1").into()
}

fn is_enclosure(str: &str) -> bool {
  let last_char = str.chars().last().unwrap_or_default();

  let enclosure_start = match last_char {
    '}' => '{',
    ']' => '[',
    _ => return false,
  };

  let found_index = str.find(enclosure_start);
  if let Some(index) = found_index {
    return str[index + 1..].contains('/');
  }

  false
}

fn is_globby(str: &str) -> bool {
  let paren_re = Regex::new(r#"\([^()]+$"#).unwrap();
  let bracket_re = Regex::new(r#"[^\\][{\[]"#).unwrap();

  if paren_re.is_match(str) {
    return true;
  }
  if str.starts_with('{') || str.starts_with('[') {
    return true;
  }
  if bracket_re.is_match(str) {
    return true;
  }

  is_glob::is_glob(str)
}

fn dirname(path: &str) -> String {
  let result = Path::new(path)
    .parent()
    .map(|path| path.to_string_lossy().to_string())
    .unwrap_or(".".to_string());
  if result.is_empty() {
    return ".".to_string();
  }
  result
}

#[cfg(test)]
mod test {
  // https://github.com/gulpjs/glob-parent/blob/main/test/index.test.js

  use std::path::Path;

  use crate::pattern::get_base_directory;

  #[test]
  fn it_should_strip_glob_magic_to_return_parent_path() {
    assert_eq!(get_base_directory("."), ".");
    assert_eq!(get_base_directory(".*"), ".");
    assert_eq!(get_base_directory("/.*"), "/");
    assert_eq!(get_base_directory("/.*/"), "/");
    assert_eq!(get_base_directory("a/.*/b"), "a");
    assert_eq!(get_base_directory("a*/.*/b"), ".");
    assert_eq!(get_base_directory("*/a/b/c"), ".");
    assert_eq!(get_base_directory("*"), ".");
    assert_eq!(get_base_directory("*/"), ".");
    assert_eq!(get_base_directory("*/*"), ".");
    assert_eq!(get_base_directory("*/*/"), ".");
    assert_eq!(get_base_directory("**"), ".");
    assert_eq!(get_base_directory("**/"), ".");
    assert_eq!(get_base_directory("**/*"), ".");
    assert_eq!(get_base_directory("**/*/"), ".");
    assert_eq!(get_base_directory("/*.js"), "/");
    assert_eq!(get_base_directory("*.js"), ".");
    assert_eq!(get_base_directory("**/*.js"), ".");
    assert_eq!(get_base_directory("{a,b}"), ".");
    assert_eq!(get_base_directory("/{a,b}"), "/");
    assert_eq!(get_base_directory("/{a,b}/"), "/");
    assert_eq!(get_base_directory("(a|b)"), ".");
    assert_eq!(get_base_directory("/(a|b)"), "/");
    assert_eq!(get_base_directory("./(a|b)"), ".");
    assert_eq!(get_base_directory("a/(b c)"), "a"); // not an extglob
    assert_eq!(get_base_directory("a/(b c)/"), "a/(b c)"); // not an extglob
    assert_eq!(get_base_directory("a/(b c)/d"), "a/(b c)"); // not an extglob
    assert_eq!(get_base_directory("path/to/*.js"), "path/to");
    assert_eq!(get_base_directory("/root/path/to/*.js"), "/root/path/to");
    assert_eq!(get_base_directory("chapter/foo [bar]/"), "chapter");
    assert_eq!(get_base_directory("path/[a-z]"), "path");
    assert_eq!(get_base_directory("[a-z]"), ".");
    assert_eq!(get_base_directory("path/{to,from}"), "path");
    assert_eq!(get_base_directory("path/(to|from)"), "path");
    assert_eq!(
      get_base_directory("path/(foo bar)/subdir/foo.*"),
      "path/(foo bar)/subdir"
    );
    assert_eq!(get_base_directory("path/!(to|from)"), "path");
    assert_eq!(get_base_directory("path/?(to|from)"), "path");
    assert_eq!(get_base_directory("path/+(to|from)"), "path");
    assert_eq!(get_base_directory("path/*(to|from)"), "path");
    assert_eq!(get_base_directory("path/@(to|from)"), "path");
    assert_eq!(get_base_directory("path/!/foo"), "path/!");
    assert_eq!(get_base_directory("path/?/foo"), "path/?");
    assert_eq!(get_base_directory("path/+/foo"), "path/+");
    assert_eq!(get_base_directory("path/*/foo"), "path");
    assert_eq!(get_base_directory("path/@/foo"), "path/@");
    assert_eq!(get_base_directory("path/!/foo/"), "path/!/foo");
    assert_eq!(get_base_directory("path/?/foo/"), "path/?/foo");
    assert_eq!(get_base_directory("path/+/foo/"), "path/+/foo");
    assert_eq!(get_base_directory("path/*/foo/"), "path");
    assert_eq!(get_base_directory("path/@/foo/"), "path/@/foo");
    assert_eq!(get_base_directory("path/**/*"), "path");
    assert_eq!(get_base_directory("path/**/subdir/foo.*"), "path");
    assert_eq!(get_base_directory("path/subdir/**/foo.js"), "path/subdir");
    assert_eq!(get_base_directory("path/!subdir/foo.js"), "path/!subdir");
    assert_eq!(get_base_directory("path/{foo,bar}/"), "path");
  }

  #[test]
  fn it_should_respect_escaped_characters() {
    assert_eq!(
      get_base_directory("path/\\*\\*/subdir/foo.*"),
      "path/**/subdir"
    );
    assert_eq!(
      get_base_directory("path/\\[\\*\\]/subdir/foo.*"),
      "path/[*]/subdir"
    );
    assert_eq!(get_base_directory("path/\\*(a|b)/subdir/foo.*"), "path");
    assert_eq!(get_base_directory("path/\\*/(a|b)/subdir/foo.*"), "path/*");
    assert_eq!(
      get_base_directory("path/\\*\\(a\\|b\\)/subdir/foo.*"),
      "path/*(a|b)/subdir"
    );
    assert_eq!(
      get_base_directory("path/\\[foo bar\\]/subdir/foo.*"),
      "path/[foo bar]/subdir"
    );
    assert_eq!(get_base_directory("path/\\[bar]/"), "path/[bar]");
    assert_eq!(get_base_directory("path/\\[bar]"), "path");
    assert_eq!(get_base_directory("[bar]"), ".");
    assert_eq!(get_base_directory("[bar]/"), ".");
    assert_eq!(get_base_directory("./\\[bar]"), ".");
    assert_eq!(get_base_directory("\\[bar]/"), "[bar]");
    assert_eq!(get_base_directory("\\!dir/*"), "!dir");
    assert_eq!(get_base_directory("[bar\\]/"), ".");
    assert_eq!(get_base_directory("path/foo \\[bar]/"), "path/foo [bar]");
    assert_eq!(get_base_directory("path/\\{foo,bar}/"), "path/{foo,bar}");
    assert_eq!(get_base_directory("\\{foo,bar}/"), "{foo,bar}");
    assert_eq!(get_base_directory("\\{foo,bar\\}/"), "{foo,bar}");
    assert_eq!(get_base_directory("{foo,bar\\}/"), ".");

    assert_eq!(get_base_directory("foo-\\(bar\\).md"), ".");
    assert_eq!(get_base_directory("\\[bar]"), ".");
    assert_eq!(get_base_directory("[bar\\]"), ".");
    assert_eq!(get_base_directory("\\{foo,bar\\}"), ".");
    assert_eq!(get_base_directory("{foo,bar\\}"), ".");
  }

  #[test]
  fn it_should_respect_glob_enclosures_with_embedded_separators() {
    assert_eq!(get_base_directory("path/{,/,bar/baz,qux}/"), "path");
    assert_eq!(
      get_base_directory("path/\\{,/,bar/baz,qux}/"),
      "path/{,/,bar/baz,qux}"
    );
    assert_eq!(
      get_base_directory("path/\\{,/,bar/baz,qux\\}/"),
      "path/{,/,bar/baz,qux}"
    );
    assert_eq!(get_base_directory("/{,/,bar/baz,qux}/"), "/");
    assert_eq!(
      get_base_directory("/\\{,/,bar/baz,qux}/"),
      "/{,/,bar/baz,qux}"
    );
    assert_eq!(get_base_directory("{,/,bar/baz,qux}"), ".");
    assert_eq!(
      get_base_directory("\\{,/,bar/baz,qux\\}"),
      "{,/,bar/baz,qux}"
    );
    assert_eq!(
      get_base_directory("\\{,/,bar/baz,qux}/"),
      "{,/,bar/baz,qux}"
    );
    assert_eq!(get_base_directory("path/foo[a\\/]/"), "path");
    assert_eq!(get_base_directory("path/foo\\[a\\/]/"), "path/foo[a\\/]");
    assert_eq!(get_base_directory("foo[a\\/]"), ".");
    assert_eq!(get_base_directory("foo\\[a\\/]"), "foo[a\\/]");
    assert_eq!(get_base_directory("path/(foo/bar|baz)"), "path");
    assert_eq!(get_base_directory("path/(foo/bar|baz)/"), "path");
    assert_eq!(
      get_base_directory("path/\\(foo/bar|baz)/"),
      "path/(foo/bar|baz)"
    );
  }

  #[test]
  fn it_should_handle_nested_braces() {
    assert_eq!(
      get_base_directory("path/{../,./,{bar,/baz\\},qux\\}/"),
      "path"
    );
    assert_eq!(
      get_base_directory("path/{../,./,\\{bar,/baz},qux}/"),
      "path"
    );
    assert_eq!(
      get_base_directory("path/\\{../,./,\\{bar,/baz\\},qux\\}/"),
      "path/{../,./,{bar,/baz},qux}"
    );
    assert_eq!(get_base_directory("{../,./,{bar,/baz\\},qux\\}/"), ".");
    assert_eq!(get_base_directory("{../,./,{bar,/baz\\},qux\\}"), ".");
    assert_eq!(get_base_directory("path/{,/,bar/{baz,qux\\}}/"), "path");
    assert_eq!(get_base_directory("path/{,/,bar/{baz,qux}\\}/"), "path");
  }

  #[test]
  fn it_should_return_parent_dirname_from_non_glob_paths() {
    assert_eq!(get_base_directory("path"), ".");
    assert_eq!(get_base_directory("path/foo"), "path");
    assert_eq!(get_base_directory("path/foo/"), "path/foo");
    assert_eq!(get_base_directory("path/foo/bar.js"), "path/foo");
  }

  #[test]
  fn it_should_respect_disabled_auto_flip_backslashes() {
    assert_eq!(get_base_directory("foo-\\(bar\\).md"), ".");
  }

  #[test]
  fn should_get_base_name() {
    assert_eq!(get_base_directory("js/*.js"), "js");
  }

  #[test]
  fn should_get_base_name_from_nested_glob() {
    assert_eq!(get_base_directory("js/**/test/*.js"), "js");
  }

  #[test]
  fn should_get_base_name_from_flat_file() {
    assert_eq!(get_base_directory("js/test/wow.js"), "js/test");
    assert_eq!(get_base_directory("js/test/wow.js"), "js/test");
  }

  #[test]
  fn should_get_base_name_from_character_class_pattern() {
    assert_eq!(get_base_directory("js/t[a-z]st}/*.js"), "js");
  }

  #[test]
  fn should_get_base_name_from_brace_expansion() {
    assert_eq!(get_base_directory("js/{src,test}/*.js"), "js");
  }

  #[test]
  fn should_get_base_name_from_brace_range_expansion() {
    assert_eq!(get_base_directory("js/test{0..9}/*.js"), "js");
  }

  #[test]
  fn should_get_base_name_from_extglob() {
    assert_eq!(get_base_directory("js/t+(wo|est)/*.js"), "js");
  }

  #[test]
  fn should_get_base_name_from_non_extglob_parens() {
    assert_eq!(get_base_directory("js/t(wo|est)/*.js"), "js");
    assert_eq!(get_base_directory("js/t/(wo|est)/*.js"), "js/t");
  }

  #[test]
  fn should_get_base_name_from_complex_brace_glob() {
    assert_eq!(
      get_base_directory("lib/{components,pages}/**/{test,another}/*.txt"),
      "lib"
    );
    assert_eq!(
      get_base_directory("js/test/**/{images,components}/*.js"),
      "js/test"
    );
    assert_eq!(
      get_base_directory("ooga/{booga,sooga}/**/dooga/{eooga,fooga}"),
      "ooga"
    );
  }

  #[test]
  fn should_not_be_susceptible_to_snyk_js_globparent_1016905() {
    // This will time out if susceptible.
    get_base_directory(&"{".repeat(500000));
  }

  //   #[test]
  //   fn should_finish_in_reasonable_time_for_repeat_patterns() {
  //     get_base_directory(&"{".repeat(500000));
  //     get_base_directory("{".repeat(500000).as_str());
  //     get_base_directory("(".repeat(500000).as_str());
  //     get_base_directory(&"/(".repeat(500000) + ")");
  //   }
}
