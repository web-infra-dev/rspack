mod reference;

use rspack_glob::{Error, ErrorKind, GlobMatch, GlobPattern, validate};

#[test]
fn test_match_path_returns_exact_partial_and_no_match() {
  let pattern = GlobPattern::new("src/{components,widgets}/*.js".as_bytes()).expect("valid glob");
  let GlobMatch::MatchPartial(remaining) = pattern.match_path("src/components/") else {
    panic!("directory prefix should return its remaining pattern");
  };
  assert!(matches!(
    remaining.match_path("index.js"),
    GlobMatch::MatchExact
  ));
  assert!(matches!(
    remaining.match_path("index.css"),
    GlobMatch::MatchPartial(_)
  ));
  assert!(matches!(pattern.match_path("other/"), GlobMatch::NotMatch));
  let GlobMatch::MatchPartial(filename) = remaining.match_path("index.") else {
    panic!("filename prefix should return a remaining pattern");
  };
  assert!(matches!(filename.match_path("js"), GlobMatch::MatchExact));
}

#[test]
fn test_exact_match_can_still_have_continuations() {
  let pattern = GlobPattern::new("{a,a/**}".as_bytes()).expect("valid glob");
  assert!(matches!(pattern.match_path("a"), GlobMatch::MatchExact));
  let remaining = pattern.match_prefix("a/").expect("directory continuation");
  assert!(matches!(
    remaining.match_path("nested/file.js"),
    GlobMatch::MatchExact
  ));
  assert!(matches!(
    GlobPattern::new("!**".as_bytes()).unwrap().match_path("a"),
    GlobMatch::NotMatch
  ));
  let negated = GlobPattern::new("!a".as_bytes()).expect("valid glob");
  let GlobMatch::MatchPartial(remaining) = negated.match_path("a") else {
    panic!("complement can still match longer paths");
  };
  assert!(remaining.match_path("b").is_exact());
}

#[test]
fn test_literal_prefix_is_derived_from_compiled_branches() {
  let pattern =
    GlobPattern::new(r"{src/\[a\]/one,src/\[a\]/two}/*.js".as_bytes()).expect("valid glob");
  assert_eq!(pattern.literal_prefix(), b"src/[a]/");
  let remaining = pattern.match_prefix("src/[a]/o").expect("literal prefix");
  assert_eq!(remaining.literal_prefix(), b"ne/");
  assert!(
    GlobPattern::new("!src/*.js".as_bytes())
      .unwrap()
      .literal_prefix()
      .is_empty()
  );
}

#[test]
fn test_literal_prefix_borrows_input_and_remaining_source() {
  let source = String::from("src/components/*.js");
  let pattern = GlobPattern::new(source.as_bytes()).expect("valid borrowed glob");
  let prefix: &[u8] = pattern.literal_prefix();
  assert_eq!(prefix, b"src/components/");
  assert!(std::ptr::eq(prefix.as_ptr(), source.as_bytes().as_ptr()));
  let remaining = pattern.match_prefix("src/").expect("matching prefix");
  assert_eq!(remaining.literal_prefix(), b"components/");
  assert!(std::ptr::eq(
    remaining.literal_prefix().as_ptr(),
    source.as_bytes()[4..].as_ptr()
  ));
  // The remaining pattern borrows the source independently of the original
  // matcher and shares its instruction graph.
  drop(pattern);
  assert!(remaining.match_path("components/index.js").is_exact());
}

#[test]
fn test_decoded_joined_and_folded_prefixes_are_shared_by_clones() {
  let joined = GlobPattern::new(br"a{b,b}c/\[x\]/*.js").expect("valid escaped glob");
  let prefix = joined.literal_prefix();
  assert_eq!(prefix, b"abc/[x]/");
  let clone = joined.clone();
  assert!(std::ptr::eq(
    prefix.as_ptr(),
    clone.literal_prefix().as_ptr()
  ));
  assert!(std::ptr::eq(
    prefix.as_ptr(),
    joined.literal_prefix().as_ptr()
  ));

  let source = String::from("Ü/SRC/*.JS");
  let folded = GlobPattern::new_with_options(
    source.as_bytes(),
    rspack_glob::GlobOptions {
      case_sensitive: false,
      ..Default::default()
    },
  )
  .expect("valid case-insensitive glob");
  assert_eq!(folded.literal_prefix(), "ü/src/".as_bytes());
  let clone = folded.clone();
  assert!(std::ptr::eq(
    folded.literal_prefix().as_ptr(),
    clone.literal_prefix().as_ptr()
  ));
  let remaining = folded.match_prefix("Ü/").expect("folded prefix");
  drop(folded);
  assert_eq!(remaining.literal_prefix(), b"src/");
  assert!(remaining.match_path("SrC/INDEX.js").is_exact());
  assert_eq!(source, "Ü/SRC/*.JS");
}

#[test]
fn test_remaining_glob_preserves_all_globstar_branches() {
  let pattern = GlobPattern::new("a/**/b/*.js".as_bytes()).expect("valid glob");
  let remaining = pattern.match_prefix("a/x/b/").expect("matching prefix");
  assert!(remaining.match_path("index.js").is_exact());
  assert!(remaining.match_path("y/b/index.js").is_exact());
  assert!(!remaining.match_path("y/index.js").is_exact());
  assert!(pattern.match_prefix("other/").is_none());
  assert!(!pattern.match_path("").is_exact());
}

#[test]
fn test_literal_alias_prefix_and_independent_siblings() {
  let pattern =
    GlobPattern::new("@/components/{button,dialog}/**/*.js".as_bytes()).expect("valid glob");
  let root = pattern.match_prefix("@/components/").expect("alias prefix");
  let button = root.match_prefix("button/").expect("button prefix");
  let dialog = root.match_prefix("dialog/").expect("dialog prefix");
  assert!(button.match_path("index.js").is_exact());
  assert!(dialog.match_path("nested/index.js").is_exact());
  assert!(root.match_prefix("missing/").is_none());
  assert!(
    pattern
      .match_path("@/components/button/index.js")
      .is_exact()
  );

  let literal = GlobPattern::new(r"src/\[legacy\]/\*/?.js".as_bytes()).expect("valid escaped glob");
  assert!(
    literal
      .match_prefix("src/[legacy]/*/")
      .expect("literal prefix")
      .match_path("a.js")
      .is_exact()
  );
}

#[test]
fn test_prefix_can_end_inside_a_segment_or_utf8() {
  let pattern = GlobPattern::new("a/**/b".as_bytes()).expect("valid glob");
  let remaining = pattern.match_prefix("a/part").expect("partial segment");
  assert!(remaining.match_path("ial/b").is_exact());
  assert!(!remaining.match_path("b").is_exact());
  let unicode = GlobPattern::new("目录/*.js".as_bytes()).expect("valid UTF-8 pattern");
  let path = "目录/文件.js".as_bytes();
  for split in 0..=path.len() {
    let remaining = unicode
      .match_prefix(&path[..split])
      .expect("matching prefix");
    assert!(remaining.match_path(&path[split..]).is_exact());
  }
}

#[test]
fn test_empty_terminal_and_negated_patterns() {
  let empty = GlobPattern::new("".as_bytes()).expect("empty glob");
  assert!(empty.match_path("").is_exact());
  assert!(empty.match_prefix("a").is_none());
  let literal = GlobPattern::new("a".as_bytes()).expect("literal glob");
  let terminal = literal.match_prefix("a").expect("complete match");
  assert!(terminal.match_path("").is_exact());
  assert!(terminal.match_prefix("/").is_none());

  let negated = GlobPattern::new("!src/**/*.js".as_bytes()).expect("negated glob");
  let outside = negated
    .match_prefix("other/")
    .expect("complement remains alive");
  assert!(outside.match_path("").is_exact());
  assert!(outside.match_path("index.js").is_exact());
  let inside = negated.match_prefix("src/").expect("matching prefix");
  assert!(!inside.match_path("index.js").is_exact());
  assert!(inside.match_path("index.css").is_exact());
  assert!(
    !GlobPattern::new("!**".as_bytes())
      .expect("negated globstar")
      .match_path("anything")
      .is_exact()
  );
  assert!(
    GlobPattern::new("!!**".as_bytes())
      .expect("double negation")
      .match_path("anything")
      .is_exact()
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
      GlobPattern::new(pattern.as_bytes()).expect_err("invalid glob"),
      Error { kind, index }
    );
    assert!(!reference::reference_match(
      format!("!{pattern}"),
      "unrelated"
    ));
  }
}

#[test]
fn test_path_parser_preserves_class_escapes_and_error_offsets() {
  let options = rspack_glob::GlobOptions {
    windows_paths: true,
    ..Default::default()
  };
  let source = br"C:\repo/[\a\]]/*.js";
  let pattern = GlobPattern::new_with_options(source, options).unwrap();
  assert_eq!(pattern.source(), br"C:/repo/[\a\]]/*.js");
  assert!(pattern.match_path("C:/repo/a/value.js").is_exact());
  assert!(pattern.match_path("C:/repo/]/value.js").is_exact());

  let source = "C:\\目录/{a,b";
  assert_eq!(
    GlobPattern::new_with_options(source.as_bytes(), options).unwrap_err(),
    Error {
      kind: ErrorKind::UnclosedBrace,
      index: source.find('{').unwrap(),
    }
  );
  let trailing = GlobPattern::new_with_options(br"C:\repo\", options).unwrap();
  assert_eq!(trailing.directory_prefix(), b"C:/repo/");
  assert!(trailing.match_path("C:/repo/").is_exact());

  // Standard glob escaping is independent of Windows-form pattern parsing.
  assert!(
    GlobPattern::new(br"{a\,b,c}")
      .unwrap()
      .match_path("a,b")
      .is_exact()
  );
  assert!(
    GlobPattern::new(br"\n")
      .unwrap()
      .match_path(b"\n")
      .is_exact()
  );
  assert!(GlobPattern::new(br"C:\repo\").is_err());
}

#[test]
fn test_negated_terminal_globstar_can_prune_a_prefix() {
  assert!(
    GlobPattern::new("!**".as_bytes())
      .expect("negated globstar")
      .match_prefix("")
      .is_none()
  );
  assert!(
    GlobPattern::new("!a/**".as_bytes())
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
    let expected = reference::reference_match(pattern, path) || retained_branch;
    assert_eq!(
      compiled.match_path(path).is_exact(),
      expected,
      "pattern {pattern:?}, path {path:?}"
    );
    for split in 0..=path.len() {
      let result = compiled.match_prefix(&path[..split]);
      assert_eq!(
        result
          .as_ref()
          .is_some_and(|remaining| remaining.match_path(&path[split..]).is_exact()),
        expected,
        "pattern {pattern:?}, path {path:?}, split {split}"
      );
      if let Some(remaining) = result {
        // Repeated consumption must preserve the same residual language.
        for next in split..=path.len() {
          let result = remaining.match_prefix(&path[split..next]);
          assert_eq!(
            result.is_some_and(|remaining| remaining.match_path(&path[next..]).is_exact()),
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
  let compiled = GlobPattern::new(pattern.as_bytes()).expect("long valid glob");
  assert!(compiled.match_path(&pattern).is_exact());
  assert!(!compiled.match_path("a/").is_exact());
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
    ..Default::default()
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
    let pattern = GlobPattern::new_with_options(pattern.as_bytes(), options).expect("valid glob");
    assert_eq!(pattern.match_path(path).is_exact(), expected);
    for (split, _) in path
      .char_indices()
      .chain(std::iter::once((path.len(), '\0')))
    {
      assert_eq!(
        pattern
          .match_prefix(&path[..split])
          .is_some_and(|rest| rest.match_path(&path[split..]).is_exact()),
        expected
      );
    }
  }
  let negated =
    GlobPattern::new_with_options("!**".as_bytes(), options).expect("valid negated glob");
  assert!(
    negated
      .match_prefix(".hidden/")
      .expect("hidden paths are in the complement")
      .match_path("value.js")
      .is_exact()
  );
}
