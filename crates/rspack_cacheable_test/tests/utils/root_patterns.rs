use rspack_cacheable::utils::root_patterns;

fn patterns(root: &str) -> Vec<String> {
  root_patterns(root)
    .patterns
    .into_iter()
    .map(|pattern| String::from_utf8(pattern).expect("root patterns are UTF-8"))
    .collect()
}

fn matches(root: &str, haystack: &str) -> bool {
  let patterns = root_patterns(root);
  patterns.bind_all
    || patterns.patterns.iter().any(|pattern| {
      haystack
        .as_bytes()
        .windows(pattern.len())
        .any(|w| w == pattern)
    })
}

#[test]
fn windows_root_forms() {
  let root = r"C:\work\project";
  for spelling in [
    r"C:\work\project\src\index.js",
    r#""C:\\work\\project\\src\\index.js""#,
    "C:/work/project/src/index.js",
    "file:///C:/work/project/src/index.js",
    r"c:\work\project\src\index.js",
    "c:/work/project/src/index.js",
    r"C:\/work\/project\/src",
  ] {
    assert!(matches(root, spelling), "{spelling}");
  }
  assert!(!matches(root, r"C:\work\other\index.js"));
  assert!(!matches(root, r"D:\work\project\index.js"));
}

#[test]
fn nested_json_strings() {
  // `JSON.stringify(JSON.stringify(path))`
  assert!(matches(
    r"C:\work\project",
    r#""\"C:\\\\work\\\\project\\\\src\\\\index.js\"""#
  ));
  assert!(matches("/work/a\"b", r#""\"/work/a\\\"b/index.js\"""#));
}

#[test]
fn json_control_character_escapes() {
  assert!(matches(
    "/tmp/project\tname",
    r#""/tmp/project\tname/src/a.js""#
  ));
  assert!(matches("/tmp/a\nproject", r#""/tmp/a\nproject/src/a.js""#));
  assert!(matches(
    "/tmp/a\u{1}project",
    r#""/tmp/a\u0001project/src/a.js""#
  ));
  assert!(!matches("/tmp/project\tname", "/tmp/project name/src/a.js"));
}

#[test]
fn escaped_solidus() {
  assert!(matches("/work/project", r"\/work\/project\/src\/a\.js"));
}

#[test]
fn regex_escaped_root() {
  let root = "/work/github.com/my-app[1]";
  for spelling in [
    // The MDN snippet, lodash `escapeRegExp` and `escape-string-regexp` 1: the escaped string,
    // `regex.toString()`, and `JSON.stringify(regex.source)`.
    r"/work/github\.com/my-app\[1\]/src/",
    r"/^\/work\/github\.com\/my-app\[1\]\/src\//",
    r#""^\\/work\\/github\\.com\\/my-app\\[1\\]\\/src\\/""#,
    // A character class that lists `-`.
    r"/work/github\.com/my\-app\[1\]/src/",
    r"/^\/work\/github\.com\/my\-app\[1\]\/src\//",
    // `escape-string-regexp` 5.
    r"/work/github\.com/my\x2dapp\[1\]/src/",
    r#""/work/github\\.com/my\\x2dapp\\[1\\]/src/""#,
    // `RegExp.escape`, which agrees with `escape-string-regexp` 5 in a regex source here.
    r"/^\/work\/github\.com\/my\x2dapp\[1\]\/src\//",
  ] {
    assert!(matches(root, spelling), "{spelling}");
  }
  assert!(!matches(root, r"/^\/work\/github\.com\/my-lib\/src\//"));
}

#[test]
fn standard_regex_escape_of_punctuation_and_white_space() {
  // `RegExp.escape(root)` in Node.js 24.
  assert!(matches(
    "/work/my app, v2",
    r"/^\/work\/my\x20app\x2c\x20v2\/src\//"
  ));
  assert!(matches(
    "/work/a\u{3000}b\tc",
    r"/^\/work\/a\u3000b\tc\/src\//"
  ));
}

#[test]
fn regex_escaped_windows_root() {
  let root = r"C:\work\my-app.v2";
  for spelling in [
    r"/^C:\\work\\my-app\.v2\\src/",
    r"/^C:\/work\/my-app\.v2\/src/",
    r"/^c:\\work\\my\x2dapp\.v2\\src/",
    // `RegExp.escape` with the root first, and with the root after other text.
    r"/\x43\x3a\\work\\my\x2dapp\.v2\\src/",
    r"/^C\x3a\\work\\my\x2dapp\.v2\\src/",
  ] {
    assert!(matches(root, spelling), "{spelling}");
  }
}

#[test]
fn regex_escaped_non_ascii_root() {
  let root = "/work/读.b";
  for spelling in [
    r"/^\/work\/读\.b\/src\//",
    // A regex literal and its JSON string content, printed with an ASCII charset.
    r"/^\/work\/\u8bfb\.b\/src\//",
    r#""^\\/work\\/\u8BFB\\.b\\/src\\/""#,
  ] {
    assert!(matches(root, spelling), "{spelling}");
  }
}

#[test]
fn windows_root_with_special_characters() {
  let root = r"C:\work\my project\ü";
  assert!(matches(
    root,
    "file:///C:/work/my%20project/%C3%BC/index.js"
  ));
  assert!(matches(root, "C:/work/my project/ü/index.js"));
  assert!(matches(root, r#""C:\\work\\my project\\ü\\index.js""#));
}

#[test]
fn windows_unc_root() {
  let root = r"\\server\share\project";
  assert!(matches(root, r"\\server\share\project\index.js"));
  assert!(matches(root, "//server/share/project/index.js"));
  assert!(matches(root, "file://server/share/project/index.js"));
}

#[test]
fn windows_unc_host_is_normalized_like_url_crate() {
  assert!(matches(
    r"\\SERVER\share\my project",
    "file://server/share/my%20project/src/a.js"
  ));
  assert!(matches(
    r"\\münich\share\my project",
    "file://xn--mnich-kva/share/my%20project/src/a.js"
  ));
  assert!(matches(
    r"\\SERVER\share\my project",
    r"\\SERVER\share\my project\src\a.js"
  ));
}

#[test]
fn trailing_separators_are_ignored() {
  assert_eq!(patterns(r"C:\work\project\"), patterns(r"C:\work\project"));
  assert_eq!(patterns("C:/work/project/"), patterns("C:/work/project"));
  assert_eq!(patterns("/work/project/"), patterns("/work/project"));
  // Even a one-byte root retains its nonempty stable base64 fragments.
  let mut found = patterns("/");
  found.sort();
  assert_eq!(found, vec!["/", "L", "v"]);
}

#[test]
fn unix_root_patterns() {
  let mut found = patterns("/work/project");
  found.sort();
  // Keep every character whose six bits come solely from the root, including
  // characters in the final incomplete three-byte group.
  assert_eq!(
    found,
    vec![
      "/work/project",
      "93b3JrL3Byb2plY3",
      "L3dvcmsvcHJvamVjd",
      r"\/work\/project",
      r"\\/work\\/project",
      "vd29yay9wcm9qZWN0",
    ]
  );
}

#[test]
fn short_root_patterns_keep_all_stable_base64_characters() {
  let mut found = patterns("/app");
  found.sort();
  // Escaping the sole leading slash still contains the literal `/app`, so the
  // escaped-solidus candidates are redundant.
  assert_eq!(found, vec!["/app", "9hcH", "L2Fwc", "vYXBw"]);
}

#[test]
fn one_byte_root_requires_binding_without_empty_patterns() {
  assert!(root_patterns("/").bind_all);
  assert!(!root_patterns("/app").bind_all);
  assert!(patterns("/").iter().all(|pattern| !pattern.is_empty()));
  for value in ["", "unrelated", "eyJmaWxlIjoiL290aGVyL2luZGV4LmpzIn0="] {
    assert!(matches("/", value), "{value}");
  }
}

#[test]
fn unix_root_with_special_characters() {
  let root = "/work/my project/a\"b/ü#?";
  for spelling in [
    "/work/my project/a\"b/ü#?/index.js",
    "\"/work/my project/a\\\"b/ü#?/index.js\"",
    "file:///work/my%20project/a%22b/%C3%BC%23%3F/index.js",
  ] {
    assert!(matches(root, spelling), "{spelling}");
  }
}

#[cfg(not(windows))]
#[test]
fn unix_url_form_matches_url_crate() {
  for root in [
    "/work/project",
    "/work/my project/ü",
    "/w/a\"b#c?d%e{f}g`h<i>j\\k",
  ] {
    let url = url::Url::from_file_path(root).unwrap();
    let expected = url.as_str().trim_start_matches("file://");
    assert!(
      patterns(root).iter().any(|p| p == expected),
      "{root}: {expected}"
    );
  }
}

#[cfg(windows)]
#[test]
fn windows_url_form_matches_url_crate() {
  for root in [
    r"C:\work\project",
    r"C:\work\my project\ü",
    r"\\server\share\project",
    r"\\SERVER\share\project",
    r"\\münich\share\project",
  ] {
    let url = url::Url::from_file_path(root).unwrap();
    let expected = url.as_str().trim_start_matches("file://");
    assert!(matches(root, expected), "{root}: {expected}");
  }
}
