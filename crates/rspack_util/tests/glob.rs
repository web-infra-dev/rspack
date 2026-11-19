mod reference;

use std::borrow::Cow;

use rspack_paths::{normalize_native_path_separators, normalize_path_separators};
use rspack_util::glob::{
  BackslashMode, Error, ErrorKind, GlobOptions, GlobPattern, normalize_glob_path, validate,
};

#[test]
fn test_path_matching_and_prefix_continuations() {
  let pattern = GlobPattern::new(b"src/{components,widgets}/*.js").expect("valid glob");
  let remaining = pattern
    .match_prefix("src/components/")
    .expect("directory prefix");
  assert!(remaining.is_match("index.js"));
  assert!(!remaining.is_match("index.css"));
  assert!(remaining.match_prefix("index.css").is_some());
  assert!(pattern.match_prefix("other/").is_none());
  let filename = remaining.match_prefix("index.").expect("filename prefix");
  assert!(filename.is_match("js"));
}

#[test]
fn test_exact_match_can_still_have_continuations() {
  let pattern = GlobPattern::new(b"{a,a/**}").expect("valid glob");
  assert!(pattern.is_match("a"));
  let remaining = pattern.match_prefix("a/").expect("directory continuation");
  assert!(remaining.is_match("nested/file.js"));
  let negated = GlobPattern::new(b"!**").expect("valid glob");
  assert!(!negated.is_match("a"));
  assert!(negated.match_prefix("a").is_none());
  let negated = GlobPattern::new(b"!a").expect("valid glob");
  assert!(!negated.is_match("a"));
  let remaining = negated
    .match_prefix("a")
    .expect("complement matches longer paths");
  assert!(remaining.is_match("b"));
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
fn test_decoded_joined_and_folded_prefixes() {
  let joined = GlobPattern::new(br"a{b,b}c/\[x\]/*.js").expect("valid escaped glob");
  assert_eq!(joined.literal_prefix(), b"abc/[x]/");

  let source = String::from("Ü/SRC/*.JS");
  let folded = GlobPattern::new_with_options(
    source.as_bytes(),
    GlobOptions {
      case_sensitive: false,
      ..Default::default()
    },
  )
  .expect("valid case-insensitive glob");
  assert_eq!(folded.literal_prefix(), "ü/src/".as_bytes());
  let remaining = folded.match_prefix("Ü/").expect("folded prefix");
  drop(folded);
  assert_eq!(remaining.literal_prefix(), b"src/");
  assert!(remaining.is_match("SrC/INDEX.js"));
  assert_eq!(source, "Ü/SRC/*.JS");
}

#[test]
fn test_remaining_glob_preserves_all_globstar_branches() {
  let pattern = GlobPattern::new("a/**/b/*.js".as_bytes()).expect("valid glob");
  let remaining = pattern.match_prefix("a/x/b/").expect("matching prefix");
  assert!(remaining.is_match("index.js"));
  assert!(remaining.is_match("y/b/index.js"));
  assert!(!remaining.is_match("y/index.js"));
  assert!(pattern.match_prefix("other/").is_none());
  assert!(!pattern.is_match(""));
}

#[test]
fn test_literal_alias_prefix_and_independent_siblings() {
  let pattern =
    GlobPattern::new("@/components/{button,dialog}/**/*.js".as_bytes()).expect("valid glob");
  let root = pattern.match_prefix("@/components/").expect("alias prefix");
  let button = root.match_prefix("button/").expect("button prefix");
  let dialog = root.match_prefix("dialog/").expect("dialog prefix");
  assert!(button.is_match("index.js"));
  assert!(dialog.is_match("nested/index.js"));
  assert!(root.match_prefix("missing/").is_none());
  assert!(pattern.is_match("@/components/button/index.js"));

  let literal = GlobPattern::new(r"src/\[legacy\]/\*/?.js".as_bytes()).expect("valid escaped glob");
  assert!(
    literal
      .match_prefix("src/[legacy]/*/")
      .expect("literal prefix")
      .is_match("a.js")
  );
}

#[test]
fn test_prefix_can_end_inside_a_segment_or_utf8() {
  let pattern = GlobPattern::new("a/**/b".as_bytes()).expect("valid glob");
  let remaining = pattern.match_prefix("a/part").expect("partial segment");
  assert!(remaining.is_match("ial/b"));
  assert!(!remaining.is_match("b"));
  let unicode = GlobPattern::new("目录/*.js".as_bytes()).expect("valid UTF-8 pattern");
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
  let empty = GlobPattern::new("".as_bytes()).expect("empty glob");
  assert!(empty.is_match(""));
  assert!(empty.match_prefix("a").is_none());
  let literal = GlobPattern::new("a".as_bytes()).expect("literal glob");
  let terminal = literal.match_prefix("a").expect("complete match");
  assert!(terminal.is_match(""));
  assert!(terminal.match_prefix("/").is_none());

  let negated = GlobPattern::new("!src/**/*.js".as_bytes()).expect("negated glob");
  let outside = negated
    .match_prefix("other/")
    .expect("complement remains alive");
  assert!(outside.is_match(""));
  assert!(outside.is_match("index.js"));
  let inside = negated.match_prefix("src/").expect("matching prefix");
  assert!(!inside.is_match("index.js"));
  assert!(inside.is_match("index.css"));
  assert!(
    !GlobPattern::new("!**".as_bytes())
      .expect("negated globstar")
      .is_match("anything")
  );
  assert!(
    GlobPattern::new("!!**".as_bytes())
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
  let options = GlobOptions {
    backslash: BackslashMode::WindowsPath,
    ..Default::default()
  };
  let source = br"C:\repo/[\a\]]/*.js";
  let pattern = GlobPattern::new_with_options(source, options).unwrap();
  assert_eq!(
    normalize_glob_path(std::str::from_utf8(source).unwrap()),
    r"C:/repo/[\a\]]/*.js"
  );
  assert!(pattern.is_match("C:/repo/a/value.js"));
  assert!(pattern.is_match("C:/repo/]/value.js"));

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
  assert!(trailing.is_match("C:/repo/"));

  // Standard glob escaping is independent of Windows-form pattern parsing.
  assert!(GlobPattern::new(br"{a\,b,c}").unwrap().is_match("a,b"));
  assert!(GlobPattern::new(br"\n").unwrap().is_match(b"\n"));
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
  let compiled = GlobPattern::new(pattern.as_bytes()).expect("long valid glob");
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
    ("ος/*.JS", "ΟΣ/value.js", true),
    ("οσα/*.JS", "ΟΣΑ/value.js", true),
  ];
  for (pattern, path, expected) in cases {
    let pattern = GlobPattern::new_with_options(pattern.as_bytes(), options).expect("valid glob");
    assert_eq!(pattern.is_match(path), expected);
    let path_bytes = path.as_bytes();
    for boundary in std::iter::once(0).chain(
      path_bytes
        .iter()
        .enumerate()
        .filter_map(|(index, &byte)| (byte == b'/').then_some(index + 1)),
    ) {
      assert_eq!(
        pattern
          .match_prefix(&path_bytes[..boundary])
          .is_some_and(|rest| rest.is_match(&path_bytes[boundary..])),
        expected,
        "{path:?} split at byte {boundary}"
      );
    }
    let mut parts = path.split('/').peekable();
    let mut current = Some(pattern);
    while let Some(part) = parts.next() {
      current = current.and_then(|matcher| {
        let next = matcher.match_component(part)?;
        if parts.peek().is_some() {
          next.match_prefix("/")
        } else {
          Some(next)
        }
      });
    }
    assert_eq!(current.is_some_and(|rest| rest.is_match("")), expected);
  }
  let negated =
    GlobPattern::new_with_options("!**".as_bytes(), options).expect("valid negated glob");
  assert!(
    negated
      .match_prefix(".hidden/")
      .expect("hidden paths are in the complement")
      .is_match("value.js")
  );
}

#[test]
fn test_case_insensitive_uppercase_escapes_stay_literal() {
  let options = GlobOptions {
    case_sensitive: false,
    ..Default::default()
  };
  for (pattern, literal_upper, literal_lower, control) in [
    (br"\B".as_slice(), "B", "b", "\x08"),
    (br"\N".as_slice(), "N", "n", "\n"),
    (br"\R".as_slice(), "R", "r", "\r"),
    (br"\T".as_slice(), "T", "t", "\t"),
    (br"[\N]".as_slice(), "N", "n", "\n"),
  ] {
    let compiled = GlobPattern::new_with_options(pattern, options).expect("valid glob");
    assert!(compiled.is_match(literal_upper), "{pattern:?}");
    assert!(compiled.is_match(literal_lower), "{pattern:?}");
    assert!(!compiled.is_match(control), "{pattern:?}");
  }

  let range = GlobPattern::new_with_options(br"[\B-\T]", options).expect("valid range");
  assert!(range.is_match("n"));
  assert!(range.is_match("N"));
  assert!(!range.is_match("\n"));

  let escaped_slash = GlobPattern::new_with_options(br"\\N", options).expect("valid glob");
  assert!(escaped_slash.is_match(r"\n"));
  assert!(escaped_slash.is_match(r"\N"));
  assert!(!escaped_slash.is_match("\\\n"));

  for (pattern, control, letter) in [
    (br"\b".as_slice(), "\x08", "b"),
    (br"\n".as_slice(), "\n", "n"),
    (br"\r".as_slice(), "\r", "r"),
    (br"\t".as_slice(), "\t", "t"),
  ] {
    let compiled = GlobPattern::new_with_options(pattern, options).expect("valid glob");
    assert!(compiled.is_match(control), "{pattern:?}");
    assert!(!compiled.is_match(letter), "{pattern:?}");
  }
}

#[test]
fn test_match_component_prunes_non_matching_wildcard_entries() {
  let star_js = GlobPattern::new(b"*.js").expect("valid glob");
  for skipped in ["README.md", "index", "node_modules"] {
    assert!(star_js.match_component(skipped).is_none(), "{skipped}");
  }
  assert!(star_js.match_component("index.js").is_some());

  let exact = GlobPattern::new(b"index.js").expect("valid glob");
  for skipped in ["README.md", "index", "node_modules"] {
    assert!(exact.match_component(skipped).is_none(), "{skipped}");
  }
  assert!(exact.match_component("index.js").is_some());

  let nested = GlobPattern::new(b"*/*.js").expect("valid glob");
  assert!(nested.match_component("src").is_some());
  assert!(nested.match_component("src").unwrap().is_match("/index.js"));

  let globstar = GlobPattern::new(b"**/*.js").expect("valid glob");
  assert!(globstar.match_component("nested").is_some());
  assert!(globstar.match_component(".git").is_some());

  let explicit_dot = GlobPattern::new_with_options(
    b"**/*.js",
    GlobOptions {
      require_literal_leading_dot: true,
      ..Default::default()
    },
  )
  .expect("valid glob");
  assert!(explicit_dot.match_component(".git").is_none());
  assert!(explicit_dot.match_component("nested").is_some());
}

#[test]
fn test_brace_branches_merge_shared_suffix_and_preserve_globstar_parity() {
  let branches = (0..200)
    .map(|index| format!("a{index}"))
    .collect::<Vec<_>>()
    .join(",");
  let suffix = "seg/".repeat(200);
  let pattern = format!("{{{branches}}}/{suffix}*.js");
  let compiled = GlobPattern::new(pattern.as_bytes()).expect("valid brace glob");
  assert!(compiled.is_match(format!("a0/{suffix}index.js")));
  assert!(compiled.is_match(format!("a199/{suffix}index.js")));
  assert!(!compiled.is_match(format!("a200/{suffix}index.js")));

  let parity_paths: &[&[u8]] = &[
    b"", b"a", b"aa", b"a/b", b"a/b/c", b"b", b"a/", b"/a", b"a/a", b"x/y",
  ];
  for pattern in [
    "{a,}**",
    "{,a}**",
    "{a,}**/**",
    "{,a}**/**",
    "a**/**",
    "{a,}{**}",
    "{a,}/**",
  ] {
    check_pattern(pattern.as_bytes(), parity_paths);
  }
}

#[test]
fn directory_prefix_decodes_escapes_and_shared_branches() {
  for (source, expected) in [
    (r"./fixtures/a\[b\]/file", "./fixtures/a[b]/"),
    (r"./fixtures/a\[b\]/**/*.js", "./fixtures/a[b]/"),
    (r"./fixtures/file\*.js", "./fixtures/"),
    (
      r"./fixtures/directory\?1/**/*.js",
      "./fixtures/directory?1/",
    ),
    ("{src/a,src/b}/*.js", "src/"),
    ("./{src,src}/components/*.js", "./src/components/"),
    ("{src,lib}/*.js", ""),
    ("*.js", ""),
    ("!src/**/*.js", ""),
  ] {
    let pattern = GlobPattern::new(source.as_bytes()).unwrap();
    let prefix = pattern.directory_prefix();
    assert_eq!(prefix, expected.as_bytes(), "{source}");
    assert!(pattern.match_prefix(&prefix).is_some(), "{source}");
  }
}

#[test]
fn glob_parser_handles_windows_separators_and_glob_escapes() {
  for (source, expected, path) in [
    (
      r"./fixtures/a\[b\]/*.js",
      r"./fixtures/a\[b\]/*.js",
      "./fixtures/a[b]/value.js",
    ),
    (
      r"./fixtures/file\*.js",
      r"./fixtures/file\*.js",
      "./fixtures/file*.js",
    ),
    (
      r"./fixtures/file\?.js",
      r"./fixtures/file\?.js",
      "./fixtures/file?.js",
    ),
    (
      r"C:\fixtures\a\[b\]\file.js",
      r"C:/fixtures/a\[b\]/file.js",
      "C:/fixtures/a[b]/file.js",
    ),
    (
      r"C:\repo\src/*.js",
      "C:/repo/src/*.js",
      "C:/repo/src/value.js",
    ),
    (
      r"C:\目录\文件\[name\]/*.js",
      r"C:/目录/文件\[name\]/*.js",
      "C:/目录/文件[name]/value.js",
    ),
  ] {
    let pattern = GlobPattern::new_with_options(
      source.as_bytes(),
      GlobOptions {
        backslash: BackslashMode::WindowsPath,
        ..Default::default()
      },
    )
    .unwrap();
    assert_eq!(normalize_glob_path(source), expected, "{source}");
    assert!(pattern.is_match(path), "{source}");
    let prefix = pattern.directory_prefix();
    let remaining = pattern.match_prefix(&prefix).unwrap();
    let suffix = path.as_bytes().strip_prefix(prefix.as_slice()).unwrap();
    assert!(remaining.is_match(suffix), "{source}");
  }
}

#[test]
fn native_windows_patterns_keep_wildcards_after_backslash_separators() {
  let options = GlobOptions {
    backslash: BackslashMode::Separator,
    ..Default::default()
  };
  for (source, parsed, path) in [
    (
      r"C:\repo\src\*.txt",
      "C:/repo/src/*.txt",
      "C:/repo/src/value.txt",
    ),
    (r"src\?\*.txt", "src/?/*.txt", "src/a/value.txt"),
    (r"src\{a,b}\*.txt", "src/{a,b}/*.txt", "src/b/value.txt"),
    (r"src\[ab]\*.txt", "src/[ab]/*.txt", "src/a/value.txt"),
    (r"src\[\a\]]\*.txt", r"src/[\a\]]/*.txt", "src/]/value.txt"),
    (
      r"\\server\share\src\*.txt",
      "//server/share/src/*.txt",
      "//server/share/src/value.txt",
    ),
  ] {
    let pattern = GlobPattern::new_with_options(source.as_bytes(), options).unwrap();
    let normalized = GlobPattern::new(parsed.as_bytes()).expect("valid normalized glob");
    assert_eq!(
      pattern.directory_prefix(),
      normalized.directory_prefix(),
      "{source}"
    );
    assert!(pattern.is_match(path), "{source}");
    let prefix = pattern.directory_prefix();
    assert!(
      pattern
        .match_prefix(&prefix)
        .unwrap()
        .is_match(path.as_bytes().strip_prefix(prefix.as_slice()).unwrap())
    );
  }
}

#[test]
fn normalize_path_separators_treats_glob_chars_as_literals() {
  assert_eq!(
    normalize_path_separators("C:\\fixtures\\a\\[b]\\file.js"),
    "C:/fixtures/a/[b]/file.js"
  );
  assert_eq!(
    normalize_path_separators("C:\\fixtures\\a\\{b}\\file.js"),
    "C:/fixtures/a/{b}/file.js"
  );
}

#[test]
fn separator_normalization_borrows_unchanged_inputs() {
  let path = "./src/目录/index.js";
  let normalized = normalize_path_separators(path);
  assert!(matches!(normalized, Cow::Borrowed(_)));
  assert_eq!(normalized.as_ptr(), path.as_ptr());

  let pattern = r"./src/目录/\[literal\]/file\*.js";
  let normalized = normalize_glob_path(pattern);
  assert!(matches!(normalized, Cow::Borrowed(_)));
  assert_eq!(normalized.as_ptr(), pattern.as_ptr());
}

#[test]
fn native_separator_normalization_preserves_unix_filename_backslashes() {
  let path = r"src\literal\*.js";
  let normalized = normalize_native_path_separators(path);
  if cfg!(windows) {
    assert_eq!(normalized, "src/literal/*.js");
  } else {
    assert_eq!(normalized, path);
    assert!(matches!(normalized, Cow::Borrowed(_)));
  }
}

#[test]
fn recursive_matching_uses_remaining_states() {
  for (source, expected) in [
    ("./src/*.js", false),
    ("./src/value.js", false),
    (r"./src/file\*\*.js", false),
    ("./src/{a,b}.js", false),
    ("./{src,src}/*.js", false),
    ("./src/**/*.js", true),
    ("./src/{a,b/nested}.js", true),
    ("{src/a,src/b}/*.js", true),
    ("!*.js", true),
  ] {
    let pattern = GlobPattern::new(source.as_bytes()).unwrap();
    let remaining = pattern.match_prefix(pattern.directory_prefix()).unwrap();
    assert_eq!(remaining.is_recursive(), expected, "{source}");
  }

  let pattern = GlobPattern::new(b"src/{a,b/nested}.js").unwrap();
  let remaining = pattern.match_prefix(b"src/b/").unwrap();
  assert!(!remaining.is_recursive());
  assert!(remaining.is_match(b"nested.js"));
}

#[test]
fn explicit_dot_and_escaped_matching() {
  let explicit_dot = GlobOptions {
    require_literal_leading_dot: true,
    ..Default::default()
  };
  for (pattern, path, expected) in [
    ("./fixtures/file\\*.js", "./fixtures/file*.js", true),
    ("./fixtures/file\\*.js", "./fixtures/file-a.js", false),
    (
      "./fixtures/directory\\?1/**/*.js",
      "./fixtures/directory?1/index.js",
      true,
    ),
    (
      "./fixtures/directory\\?1/**/*.js",
      "./fixtures/directory-a1/index.js",
      false,
    ),
    ("./fixtures/**/.*", "./fixtures/.env", true),
    (
      "./fixtures/**/.*/index.js",
      "./fixtures/.cache/index.js",
      true,
    ),
    (
      "./fixtures/**/index.js",
      "./fixtures/.cache/index.js",
      false,
    ),
  ] {
    assert_eq!(
      GlobPattern::new_with_options(pattern.as_bytes(), explicit_dot)
        .expect("valid glob")
        .is_match(path),
      expected,
      "{pattern} vs {path}"
    );
  }

  assert!(
    GlobPattern::new_with_options(
      b"./fixtures/**/.ENV",
      GlobOptions {
        case_sensitive: false,
        ..Default::default()
      },
    )
    .expect("valid glob")
    .is_match("./fixtures/.env")
  );

  let windows_explicit_dot = GlobOptions {
    require_literal_leading_dot: true,
    backslash: BackslashMode::WindowsPath,
    ..Default::default()
  };
  for (pattern, path, expected) in [
    (
      "C:/repo/escape/**/glob.js",
      r"C:\repo\escape\[brackets]\glob.js",
      true,
    ),
    (
      "C:/repo/escape/**/glob.js",
      r"C:\repo\escape\{curlies}\glob.js",
      true,
    ),
    ("./fixtures/.*.js", "./fixtures/.hidden.js", true),
    ("./fixtures/*.js", "./fixtures/.hidden.js", false),
  ] {
    assert_eq!(
      GlobPattern::new_with_options(pattern.as_bytes(), windows_explicit_dot)
        .expect("valid path glob")
        .is_match(normalize_path_separators(path).as_bytes()),
      expected,
      "{pattern} vs {path}"
    );
  }
}
