pub fn glob_to_regexp(glob: &str, opts: &GlobOptions) -> regex::Regex {
  let mut re_str = String::new();
  let mut in_group = false;

  let mut i = 0;
  let chars = glob.chars().collect::<Vec<_>>();
  while i < chars.len() {
    let c = chars[i];
    match c {
      '/' | '$' | '^' | '+' | '.' | '(' | ')' | '=' | '!' | '|' => {
        re_str.push_str(&format!("\\{}", c));
      }

      '?' => {
        if opts.extended {
          re_str.push('.');
        } else {
          re_str.push_str(&format!("\\{}", c));
        }
      }

      '[' | ']' => {
        if opts.extended {
          re_str.push(c);
        } else {
          re_str.push_str(&format!("\\{}", c));
        }
      }

      '{' => {
        if opts.extended {
          in_group = true;
          re_str.push('(');
        } else {
          re_str.push_str(&format!("\\{}", c));
        }
      }

      '}' => {
        if opts.extended {
          in_group = false;
          re_str.push(')');
        } else {
          re_str.push_str(&format!("\\{}", c));
        }
      }

      ',' => {
        if in_group {
          re_str.push('|');
        } else {
          re_str.push_str(&format!("\\{}", c));
        }
      }

      '*' => {
        let prev_char = glob.chars().nth(i.wrapping_sub(1));
        let mut star_count = 1;
        while i + 1 < chars.len() && chars[i + 1] == '*' {
          star_count += 1;
          i += 1;
        }
        let next_char = glob.chars().nth(i + 1);

        if !opts.globstar {
          re_str.push_str(".*");
        } else {
          let is_globstar = star_count > 1
            && (prev_char == Some('/') || prev_char.is_none())
            && (next_char == Some('/') || next_char.is_none());

          if is_globstar {
            re_str.push_str("((?:[^/]*(?:\\/|$))*)");
            // move over the "/"
            if next_char == Some('/') {
              i += 1;
            }
          } else {
            re_str.push_str("([^/]*)");
          }
        }
      }

      _ => re_str.push(c),
    }
    i += 1;
  }

  if !opts.flags.contains('g') {
    re_str = format!("^{}$", re_str);
  }

  regex::Regex::new(&re_str).unwrap_or_else(|err| {
    panic!(
      "Failed to create a regex pattern from '{}': {:?}, this origin glob is '{}'",
      re_str, err, glob
    );
  })
}

#[derive(Default)]
pub struct GlobOptions {
  pub extended: bool,
  pub globstar: bool,
  pub flags: String,
}

#[cfg(test)]
mod test {
  use super::*;

  fn assert_match(glob: &str, text: &str, opts: &GlobOptions) {
    let re = glob_to_regexp(glob, opts);
    assert!(re.is_match(text));
  }

  fn assert_not_match(glob: &str, text: &str, opts: &GlobOptions) {
    let re = glob_to_regexp(glob, opts);
    assert!(!re.is_match(text));
  }

  fn test(globstar: bool) {
    // Match everything
    assert_match("*", "foo", &GlobOptions::default());
    assert_match(
      "*",
      "foo",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // Match the end
    assert_match("f*", "foo", &GlobOptions::default());
    assert_match(
      "f*",
      "foo",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // Match the start
    assert_match("*o", "foo", &GlobOptions::default());
    assert_match(
      "*o",
      "foo",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // Match the middle
    assert_match("f*uck", "firetruck", &GlobOptions::default());
    assert_match(
      "f*uck",
      "firetruck",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // Don't match without Regexp 'g'
    assert_not_match("uc", "firetruck", &GlobOptions::default());
    // Match anywhere with RegExp 'g'
    assert_match(
      "uc",
      "firetruck",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // Match zero characters
    assert_match("f*uck", "fuck", &GlobOptions::default());
    assert_match(
      "f*uck",
      "fuck",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // More complex matches
    assert_match(
      "*.min.js",
      "http://example.com/jquery.min.js",
      &GlobOptions {
        globstar: false,
        ..Default::default()
      },
    );
    assert_match(
      "*.min.*",
      "http://example.com/jquery.min.js",
      &GlobOptions {
        globstar: false,
        ..Default::default()
      },
    );
    assert_match(
      "*/js/*.js",
      "http://example.com/js/jquery.min.js",
      &GlobOptions {
        globstar: false,
        ..Default::default()
      },
    );

    // More complex matches with RegExp 'g' flag (complex regression)
    assert_match(
      "*.min.*",
      "http://example.com/jquery.min.js",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_match(
      "*.min.js",
      "http://example.com/jquery.min.js",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_match(
      "*/js/*.js",
      "http://example.com/js/jquery.min.js",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // Test string  "\\\\/$^+?.()=!|{},[].*"  represents  <glob>\\/$^+?.()=!|{},[].*</glob>
    // The equivalent regex is:  /^\\\/\$\^\+\?\.\(\)\=\!\|\{\}\,\[\]\..*$/
    // Both glob and regex match:  \/$^+?.()=!|{},[].*
    let test_str = "\\\\/$^+?.()=!|{},[].*";
    let target_str = "\\/$^+?.()=!|{},[].*";
    assert_match(test_str, target_str, &GlobOptions::default());
    assert_match(
      test_str,
      target_str,
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // Equivalent matches without/with using RegExp 'g'
    assert_not_match(
      ".min.",
      "http://example.com/jquery.min.js",
      &GlobOptions::default(),
    );
    assert_match(
      "*.min.*",
      "http://example.com/jquery.min.js",
      &GlobOptions::default(),
    );
    assert_match(
      ".min.",
      "http://example.com/jquery.min.js",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    assert_not_match(
      "http:",
      "http://example.com/jquery.min.js",
      &GlobOptions::default(),
    );
    assert_match(
      "http:*",
      "http://example.com/jquery.min.js",
      &GlobOptions::default(),
    );
    assert_match(
      "http:",
      "http://example.com/jquery.min.js",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    assert_not_match(
      "min.js",
      "http://example.com/jquery.min.js",
      &GlobOptions::default(),
    );
    assert_match(
      "*.min.js",
      "http://example.com/jquery.min.js",
      &GlobOptions::default(),
    );
    assert_match(
      "min.js",
      "http://example.com/jquery.min.js",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // Match anywhere (globally) using RegExp 'g'
    assert_match(
      "min",
      "http://example.com/jquery.min.js",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_match(
      "/js/",
      "http://example.com/js/jquery.min.js",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    assert_not_match(
      "/js*jq*.js",
      "http://example.com/js/jquery.min.js",
      &GlobOptions::default(),
    );
    assert_match(
      "/js*jq*.js",
      "http://example.com/js/jquery.min.js",
      &GlobOptions {
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // Extended mode

    // ?: Match one character, no more and no less
    assert_match(
      "f?o",
      "foo",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "f?o",
      "fooo",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "f?oo",
      "foo",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );

    // ?: Match one character with RegExp 'g'
    assert_match(
      "f?o",
      "foo",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_match(
      "f?o",
      "fooo",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_match(
      "f?o?",
      "fooo",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_not_match(
      "?fo",
      "fooo",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_not_match(
      "f?oo",
      "foo",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_not_match(
      "foo?",
      "foo",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // []: Match a character range
    assert_match(
      "fo[oz]",
      "foo",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_match(
      "fo[oz]",
      "foz",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "fo[oz]",
      "fog",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );

    // []: Match a character range and RegExp 'g' (regresion)
    assert_match(
      "fo[oz]",
      "foo",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_match(
      "fo[oz]",
      "foz",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_not_match(
      "fo[oz]",
      "fog",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // {}: Match a choice of different substrings
    assert_match(
      "foo{bar,baaz}",
      "foobaaz",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_match(
      "foo{bar,baaz}",
      "foobar",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "foo{bar,baaz}",
      "foobuzz",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_match(
      "foo{bar,b*z}",
      "foobuzz",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );

    // {}: Match a choice of different substrings and RegExp 'g' (regression)
    assert_match(
      "foo{bar,baaz}",
      "foobaaz",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "".to_string(),
        ..Default::default()
      },
    );
    assert_match(
      "foo{bar,baaz}",
      "foobar",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_not_match(
      "foo{bar,baaz}",
      "foobuzz",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_match(
      "foo{bar,b*z}",
      "foobuzz",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // More complex extended matches
    assert_match(
      "http://?o[oz].b*z.com/{*.js,*.html}",
      "http://foo.baaz.com/jquery.min.js",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_match(
      "http://?o[oz].b*z.com/{*.js,*.html}",
      "http://moz.buzz.com/index.html",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "http://?o[oz].b*z.com/{*.js,*.html}",
      "http://moz.buzz.com/index.htm",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "http://?o[oz].b*z.com/{*.js,*.html}",
      "http://moz.bar.com/index.html",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "http://?o[oz].b*z.com/{*.js,*.html}",
      "http://flozz.buzz.com/index.html",
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );

    // More complex extended matches and RegExp 'g' (regresion)
    assert_match(
      "http://?o[oz].b*z.com/{*.js,*.html}",
      "http://foo.baaz.com/jquery.min.js",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_match(
      "http://?o[oz].b*z.com/{*.js,*.html}",
      "http://moz.buzz.com/index.html",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_not_match(
      "http://?o[oz].b*z.com/{*.js,*.html}",
      "http://moz.buzz.com/index.htm",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_not_match(
      "http://?o[oz].b*z.com/{*.js,*.html}",
      "http://moz.bar.com/index.html",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_not_match(
      "http://?o[oz].b*z.com/{*.js,*.html}",
      "http://flozz.buzz.com/index.html",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // globstar
    assert_match(
      "http://foo.com/**/{*.js,*.html}",
      "http://foo.com/bar/jquery.min.js",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_match(
      "http://foo.com/**/{*.js,*.html}",
      "http://foo.com/bar/baz/jquery.min.js",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
    assert_match(
      "http://foo.com/**",
      "http://foo.com/bar/baz/jquery.min.js",
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );

    // Remaining special chars should still match themselves
    // Test string  "\\\\/$^+.()=!|,.*"  represents  <glob>\\/$^+.()=!|,.*</glob>
    // The equivalent regex is:  /^\\\/\$\^\+\.\(\)\=\!\|\,\..*$/
    // Both glob and regex match:  \/$^+.()=!|,.*
    let test_ext_str = "\\\\/$^+.()=!|,.*";
    let target_ext_str = "\\/$^+.()=!|,.*";
    assert_match(
      test_ext_str,
      target_ext_str,
      &GlobOptions {
        extended: true,
        ..Default::default()
      },
    );
    assert_match(
      test_ext_str,
      target_ext_str,
      &GlobOptions {
        extended: true,
        globstar,
        flags: "g".to_string(),
        ..Default::default()
      },
    );
  }

  #[test]
  fn test_globstar_false() {
    test(false);
  }

  #[test]
  fn test_globstar_true() {
    test(true);
  }

  #[test]
  fn test_globstar_specific_cases() {
    assert_match(
      "/foo/*",
      "/foo/bar.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "/foo/**",
      "/foo/baz.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "/foo/**",
      "/foo/bar/baz.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "/foo/*/*.txt",
      "/foo/bar/baz.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "/foo/**/*.txt",
      "/foo/bar/baz.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "/foo/**/*.txt",
      "/foo/bar/baz/qux.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "/foo/**/bar.txt",
      "/foo/bar.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "/foo/**/**/bar.txt",
      "/foo/bar.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "/foo/**/*/baz.txt",
      "/foo/bar/baz.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "/foo/**/*.txt",
      "/foo/bar.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "/foo/**/**/*.txt",
      "/foo/bar.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "/foo/**/*/*.txt",
      "/foo/bar/baz.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "**/*.txt",
      "/foo/bar/baz/qux.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "**/foo.txt",
      "foo.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "**/*.txt",
      "foo.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );

    assert_not_match(
      "/foo/*",
      "/foo/bar/baz.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "/foo/*.txt",
      "/foo/bar/baz.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "/foo/*/*.txt",
      "/foo/bar/baz/qux.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "/foo/*/bar.txt",
      "/foo/bar.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "/foo/*/*/baz.txt",
      "/foo/bar/baz.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "/foo/**.txt",
      "/foo/bar/baz/qux.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "/foo/bar**/*.txt",
      "/foo/bar/baz/qux.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "/foo/bar**",
      "/foo/bar/baz.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "**/.txt",
      "/foo/bar/baz/qux.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "*/*.txt",
      "/foo/bar/baz/qux.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "*/*.txt",
      "foo.txt",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );

    assert_not_match(
      "http://foo.com/*",
      "http://foo.com/bar/baz/jquery.min.js",
      &GlobOptions {
        extended: true,
        globstar: true,
        ..Default::default()
      },
    );
    assert_not_match(
      "http://foo.com/*",
      "http://foo.com/bar/baz/jquery.min.js",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );

    assert_match(
      "http://foo.com/*",
      "http://foo.com/bar/baz/jquery.min.js",
      &GlobOptions {
        globstar: false,
        ..Default::default()
      },
    );
    assert_match(
      "http://foo.com/**",
      "http://foo.com/bar/baz/jquery.min.js",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );

    assert_match(
      "http://foo.com/*/*/jquery.min.js",
      "http://foo.com/bar/baz/jquery.min.js",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "http://foo.com/**/jquery.min.js",
      "http://foo.com/bar/baz/jquery.min.js",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
    assert_match(
      "http://foo.com/*/*/jquery.min.js",
      "http://foo.com/bar/baz/jquery.min.js",
      &GlobOptions {
        globstar: false,
        ..Default::default()
      },
    );
    assert_match(
      "http://foo.com/*/jquery.min.js",
      "http://foo.com/bar/baz/jquery.min.js",
      &GlobOptions {
        globstar: false,
        ..Default::default()
      },
    );
    assert_not_match(
      "http://foo.com/*/jquery.min.js",
      "http://foo.com/bar/baz/jquery.min.js",
      &GlobOptions {
        globstar: true,
        ..Default::default()
      },
    );
  }
}
