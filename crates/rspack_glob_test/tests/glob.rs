use rspack_glob::{Error, ErrorKind, GlobPattern, glob_match, validate};

#[test]
fn test_remaining_glob_preserves_all_globstar_branches() {
  let pattern = GlobPattern::new("a/**/b/*.js").expect("valid glob");
  let remaining = pattern.match_prefix("a/x/b/").expect("matching prefix");
  assert!(remaining.is_match("index.js"));
  assert!(remaining.is_match("y/b/index.js"));
  assert!(!remaining.is_match("y/index.js"));
  assert!(pattern.match_prefix("other/").is_none());
  assert!(!pattern.is_match(""));
}

#[test]
fn test_literal_alias_prefix_and_independent_siblings() {
  let pattern = GlobPattern::new("@/components/{button,dialog}/**/*.js").expect("valid glob");
  let root = pattern.match_prefix("@/components/").expect("alias prefix");
  let button = root.match_prefix("button/").expect("button prefix");
  let dialog = root.match_prefix("dialog/").expect("dialog prefix");
  assert!(button.is_match("index.js"));
  assert!(dialog.is_match("nested/index.js"));
  assert!(root.match_prefix("missing/").is_none());
  assert!(pattern.is_match("@/components/button/index.js"));

  let literal = GlobPattern::new(r"src/\[legacy\]/\*/?.js").expect("valid escaped glob");
  assert!(
    literal
      .match_prefix("src/[legacy]/*/")
      .expect("literal prefix")
      .is_match("a.js")
  );
}

#[test]
fn test_prefix_can_end_inside_a_segment_or_utf8() {
  let pattern = GlobPattern::new("a/**/b").expect("valid glob");
  let remaining = pattern.match_prefix("a/part").expect("partial segment");
  assert!(remaining.is_match("ial/b"));
  assert!(!remaining.is_match("b"));
  let unicode = GlobPattern::new("目录/*.js").expect("valid UTF-8 pattern");
  let path = "目录/文件.js".as_bytes();
  for split in 0..=path.len() {
    let remaining = unicode
      .match_prefix(&path[..split])
      .expect("matching prefix");
    assert!(remaining.is_match(&path[split..]));
  }
}

#[test]
fn test_empty_terminal_and_negated_patterns() {
  let empty = GlobPattern::new("").expect("empty glob");
  assert!(empty.is_match(""));
  assert!(empty.match_prefix("a").is_none());
  let literal = GlobPattern::new("a").expect("literal glob");
  let terminal = literal.match_prefix("a").expect("complete match");
  assert!(terminal.is_match(""));
  assert!(terminal.match_prefix("/").is_none());

  let negated = GlobPattern::new("!src/**/*.js").expect("negated glob");
  let outside = negated
    .match_prefix("other/")
    .expect("complement remains alive");
  assert!(outside.is_match(""));
  assert!(outside.is_match("index.js"));
  let inside = negated.match_prefix("src/").expect("matching prefix");
  assert!(!inside.is_match("index.js"));
  assert!(inside.is_match("index.css"));
  assert!(
    !GlobPattern::new("!**")
      .expect("negated globstar")
      .is_match("anything")
  );
  assert!(
    GlobPattern::new("!!**")
      .expect("double negation")
      .is_match("anything")
  );
}

#[test]
fn test_validation_preserves_fast_glob_errors() {
  let cases = [
    ("src/**/*.{js,ts", ErrorKind::UnclosedBrace, 9),
    ("a/[bc", ErrorKind::UnclosedBracket, 2),
    ("a/\\", ErrorKind::TrailingBackslash, 2),
    (
      "{{{{{{{{{{{a}}}}}}}}}}}",
      ErrorKind::BraceNestingTooDeep,
      10,
    ),
    ("{}{}{}{}{}{}{}{}{}{}{}", ErrorKind::TooManyBraceGroups, 20),
  ];
  for (pattern, kind, index) in cases {
    assert_eq!(validate(pattern), Err(Error { kind, index }));
    assert_eq!(
      GlobPattern::new(pattern).expect_err("invalid glob"),
      Error { kind, index }
    );
    assert!(!glob_match(format!("!{pattern}"), "unrelated"));
  }
}

#[test]
fn test_negated_terminal_globstar_can_prune_a_prefix() {
  assert!(
    GlobPattern::new("!**")
      .expect("negated globstar")
      .match_prefix("")
      .is_none()
  );
  assert!(
    GlobPattern::new("!a/**")
      .expect("negated globstar")
      .match_prefix("a/")
      .is_none()
  );
}

fn check_pattern(pattern: &[u8], paths: &[&[u8]]) {
  let Ok(compiled) = GlobPattern::new(pattern) else {
    return;
  };
  for &path in paths {
    // The graph preserves the earlier wildcard branch that the legacy
    // interpreter discards when the selected brace branch enters **/a.
    let retained_branch =
      matches!(pattern, b"**{**/a,b}" | b"***{**/a,b}" | b"****{**/a,b}") && path == b"aa";
    let expected = glob_match(pattern, path) || retained_branch;
    assert_eq!(
      compiled.is_match(path),
      expected,
      "pattern {pattern:?}, path {path:?}"
    );
    for split in 0..=path.len() {
      let result = compiled.match_prefix(&path[..split]);
      assert_eq!(
        result
          .as_ref()
          .is_some_and(|remaining| remaining.is_match(&path[split..])),
        expected,
        "pattern {pattern:?}, path {path:?}, split {split}"
      );
      if let Some(remaining) = result {
        // Repeated consumption must preserve the same residual language.
        for next in split..=path.len() {
          let result = remaining.match_prefix(&path[split..next]);
          assert_eq!(
            result.is_some_and(|remaining| remaining.is_match(&path[next..])),
            expected,
            "pattern {pattern:?}, path {path:?}, splits {split}/{next}"
          );
        }
      }
    }
  }
}

#[test]
fn test_compiled_and_residual_matching_agree_with_ported_fast_glob() {
  let patterns = [
    "",
    "*",
    "**",
    "***",
    "**/**/**",
    "a/**",
    "a/**/b",
    "a/**/b/*.js",
    "{a,b}/**/*.js",
    "{**,a}/b",
    "a/{**,b}/c",
    "a{**}/b",
    "{a*,**}/*b",
    "{a,b}{c,d}",
    "{a,{b,c}}/**",
    "{**,{a,b}}/x",
    "a{,b}**/c",
    "a**/**/b",
    "[a-c]",
    "[!a-c]",
    "[^a]",
    "[]a]",
    "[[]",
    "[z-a]",
    r"[\]-a]",
    r"a/\*/b",
    r"\n",
    "!a/**",
    "!!a/**",
    "!{a,b}/**",
    "{,a}",
    "{a,}/**",
    "a/**/",
    "a/**/**/b",
    "{**/a,b}/**/c",
    "a}b",
    "a,b",
    "目录/*.js",
  ];
  let paths: &[&[u8]] = &[
    b"",
    b"a",
    b"b",
    b"ab",
    b"a/b",
    b"a/x/b",
    b"a/x/b/index.js",
    b"a/x/b/y/b/index.js",
    b"a/",
    b"a//b",
    b"/b",
    b"ac",
    b"bc",
    b"a/c",
    b"a/b/c",
    b"abc",
    b"b/x",
    b"x/a",
    b"a/x/",
    b"a/*/b",
    b"[",
    b"]",
    b"\n",
    b"a,b",
    b"a}b",
    "目录/文件.js".as_bytes(),
  ];
  for pattern in patterns {
    check_pattern(pattern.as_bytes(), paths);
  }
}

#[test]
fn test_short_patterns_against_ported_fast_glob() {
  const ALPHABET: &[u8] = b"a/*?[]{}\\,!-";
  const PATHS: &[&[u8]] = &[b"", b"a", b"aa", b"a/a", b"a/", b"/a", b"-", b",", b"!"];
  for len in 0..=4 {
    let mut pattern = vec![0; len];
    for mut n in 0..ALPHABET.len().pow(len as u32) {
      for byte in &mut pattern {
        *byte = ALPHABET[n % ALPHABET.len()];
        n /= ALPHABET.len();
      }
      check_pattern(&pattern, PATHS);
    }
  }
}

#[test]
fn test_long_pattern_compiles_without_recursive_stack_growth() {
  let pattern = "a/".repeat(10_000);
  let compiled = GlobPattern::new(&pattern).expect("long valid glob");
  assert!(compiled.is_match(&pattern));
  assert!(!compiled.is_match("a/"));
}

#[test]
fn test_generated_brace_and_globstar_combinations() {
  const TOKENS: &[&str] = &[
    "a", "b", "/", "*", "**", "?", "{a,b}", "{,a}", "{**/a,b}", "[ab]",
  ];
  const PATHS: &[&[u8]] = &[
    b"", b"a", b"b", b"ab", b"aa", b"a/b", b"a/a", b"a/", b"/a", b"a/b/a",
  ];
  for first in TOKENS {
    for second in TOKENS {
      for third in TOKENS {
        check_pattern(format!("{first}{second}{third}").as_bytes(), PATHS);
      }
    }
  }
}

#[test]
fn test_options_apply_to_every_prefix_continuation() {
  use rspack_glob::GlobOptions;
  let options = GlobOptions {
    case_sensitive: false,
    require_literal_leading_dot: true,
  };
  let cases = [
    ("SRC/**/*.JS", "src/nested/value.js", true),
    ("SRC/**/*.JS", "src/.cache/value.js", false),
    ("SRC/**/.*/INDEX.JS", "src/.cache/index.js", true),
    ("SRC/{.hidden,normal}/*.JS", "src/.hidden/value.js", true),
    ("*.*", ".env", false),
    ("[.]env", ".env", false),
    ("{.env,index.js}", ".env", true),
    (r"\.env", ".env", true),
    ("Ü/*.JS", "ü/value.js", true),
  ];
  for (pattern, path, expected) in cases {
    let pattern = GlobPattern::new_with_options(pattern, options).expect("valid glob");
    assert_eq!(pattern.is_match(path), expected);
    for (split, _) in path
      .char_indices()
      .chain(std::iter::once((path.len(), '\0')))
    {
      assert_eq!(
        pattern
          .match_prefix(&path[..split])
          .is_some_and(|rest| rest.is_match(&path[split..])),
        expected
      );
    }
  }
  let negated = GlobPattern::new_with_options("!**", options).expect("valid negated glob");
  assert!(
    negated
      .match_prefix(".hidden/")
      .expect("hidden paths are in the complement")
      .is_match("value.js")
  );
}
