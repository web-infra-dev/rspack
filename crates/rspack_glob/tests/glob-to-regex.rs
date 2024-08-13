use rspack_glob::{MatchOptions, Pattern};

#[inline]
fn glob(pat: &str, path: &str) -> bool {
  #[allow(clippy::unwrap_used)]
  let p = Pattern::new(pat).unwrap();
  p.matches_with(
    path,
    MatchOptions {
      case_sensitive: false,
      require_literal_separator: true,
      require_literal_leading_dot: false,
    },
  )
}

fn assert_match(pat: &str, text: &str) {
  assert!(glob(pat, text));
}

fn assert_not_match(pat: &str, text: &str) {
  assert!(!glob(pat, text));
}

#[test]
fn test() {
  // Match everything
  assert_match("*", "foo");

  // Match the end
  assert_match("f*", "foo");

  // Match the start
  assert_match("*o", "foo");

  // Match the middle
  assert_match("f*uck", "firetruck");

  // Don't match without Regexp 'g'
  assert_not_match("uc", "firetruck");

  // Match zero characters
  assert_match("f*uck", "fuck");

  // Equivalent matches without/with using RegExp 'g'
  assert_not_match(".min.", "http://example.com/jquery.min.js");

  assert_not_match("http:", "http://example.com/jquery.min.js");

  assert_not_match("min.js", "http://example.com/jquery.min.js");

  assert_not_match("/js*jq*.js", "http://example.com/js/jquery.min.js");
}

#[test]
fn test_globstar_specific_cases() {
  assert_match("/foo/*", "/foo/bar.txt");
  assert_match("/foo/**", "/foo/baz.txt");
  assert_match("/foo/**", "/foo/bar/baz.txt");
  assert_match("/foo/*/*.txt", "/foo/bar/baz.txt");
  assert_match("/foo/**/*.txt", "/foo/bar/baz.txt");
  assert_match("/foo/**/*.txt", "/foo/bar/baz/qux.txt");
  assert_match("/foo/**/bar.txt", "/foo/bar.txt");
  assert_match("/foo/**/**/bar.txt", "/foo/bar.txt");
  assert_match("/foo/**/*/baz.txt", "/foo/bar/baz.txt");
  assert_match("/foo/**/*.txt", "/foo/bar.txt");
  assert_match("/foo/**/**/*.txt", "/foo/bar.txt");
  assert_match("/foo/**/*/*.txt", "/foo/bar/baz.txt");
  assert_match("**/*.txt", "/foo/bar/baz/qux.txt");
  assert_match("**/foo.txt", "foo.txt");
  assert_match("**/*.txt", "foo.txt");

  assert_not_match("/foo/*", "/foo/bar/baz.txt");
  assert_not_match("/foo/*.txt", "/foo/bar/baz.txt");
  assert_not_match("/foo/*/*.txt", "/foo/bar/baz/qux.txt");
  assert_not_match("/foo/*/bar.txt", "/foo/bar.txt");
  assert_not_match("/foo/*/*/baz.txt", "/foo/bar/baz.txt");
  assert_not_match("/foo/**.txt", "/foo/bar/baz/qux.txt");
  assert_not_match("/foo/bar**/*.txt", "/foo/bar/baz/qux.txt");
  assert_not_match("/foo/bar**", "/foo/bar/baz.txt");
  assert_not_match("**/.txt", "/foo/bar/baz/qux.txt");
  assert_not_match("*/*.txt", "/foo/bar/baz/qux.txt");
  assert_not_match("*/*.txt", "foo.txt");
  assert_not_match("http://foo.com/*", "http://foo.com/bar/baz/jquery.min.js");

  assert_match("http://foo.com/**", "http://foo.com/bar/baz/jquery.min.js");

  assert_match(
    "http://foo.com/*/*/jquery.min.js",
    "http://foo.com/bar/baz/jquery.min.js",
  );
  assert_match(
    "http://foo.com/**/jquery.min.js",
    "http://foo.com/bar/baz/jquery.min.js",
  );
  assert_not_match(
    "http://foo.com/*/jquery.min.js",
    "http://foo.com/bar/baz/jquery.min.js",
  );
}
