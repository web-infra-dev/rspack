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
  assert_eq!(patterns("/"), vec!["/"]);
}

#[test]
fn unix_root_patterns() {
  let mut found = patterns("/work/project");
  found.sort();
  assert_eq!(found, vec!["/work/project", r"\/work\/project"]);
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
