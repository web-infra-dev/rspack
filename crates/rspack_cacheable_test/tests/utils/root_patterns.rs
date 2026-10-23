use rspack_cacheable::utils::root_patterns;

fn patterns(root: &str) -> Vec<String> {
  root_patterns(root)
    .into_iter()
    .map(|pattern| String::from_utf8(pattern).expect("root patterns are UTF-8"))
    .collect()
}

fn matches(root: &str, haystack: &str) -> bool {
  root_patterns(root).iter().any(|pattern| {
    haystack
      .as_bytes()
      .windows(pattern.len())
      .any(|w| w == pattern)
  })
}

#[test]
fn windows_root_forms() {
  let root = r"C:\work\project";
  let mut found = patterns(root);
  found.sort();
  assert_eq!(
    found,
    vec![r"C:/work/project", r"C:\\work\\project", r"C:\work\project"]
  );

  for spelling in [
    r"C:\work\project\src\index.js",
    r#""C:\\work\\project\\src\\index.js""#,
    "C:/work/project/src/index.js",
    "file:///C:/work/project/src/index.js",
  ] {
    assert!(matches(root, spelling), "{spelling}");
  }
  assert!(!matches(root, r"C:\work\other\index.js"));
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
fn trailing_separators_are_ignored() {
  assert_eq!(patterns(r"C:\work\project\"), patterns(r"C:\work\project"));
  assert_eq!(patterns("C:/work/project/"), patterns("C:/work/project"));
  assert_eq!(patterns("/work/project/"), patterns("/work/project"));
  assert_eq!(patterns("/"), vec!["/"]);
}

#[test]
fn unix_root_collapses_to_one_pattern() {
  assert_eq!(patterns("/work/project"), vec!["/work/project"]);
}

#[test]
fn unix_root_with_special_characters() {
  let root = "/work/my project/a\"b/ü#?";
  let mut found = patterns(root);
  found.sort();
  assert_eq!(
    found,
    vec![
      "/work/my project/a\"b/ü#?",
      "/work/my project/a\\\"b/ü#?",
      "/work/my%20project/a%22b/%C3%BC%23%3F",
    ]
  );
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
  ] {
    let url = url::Url::from_file_path(root).unwrap();
    let expected = url.as_str().trim_start_matches("file://");
    assert!(matches(root, expected), "{root}: {expected}");
  }
}
