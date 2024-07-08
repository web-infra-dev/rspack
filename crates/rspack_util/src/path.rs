use std::path::{Path, PathBuf};

pub fn relative(from: &Path, to: &Path) -> PathBuf {
  if from == to {
    return PathBuf::new();
  }

  let mut from_iter = from.components();
  let mut to_iter = to.components();
  let mut common_parts = 0;

  let from_remain = loop {
    match (from_iter.next(), to_iter.next()) {
      (Some(from_part), Some(to_part)) if from_part == to_part => {
        common_parts += 1;
      }
      (None, _) => break 0,
      _ => break from_iter.count() + 1,
    }
  };

  let mut result = PathBuf::new();
  for _ in 0..from_remain {
    result.push("..");
  }

  let to_iter = to.components().skip(common_parts);
  for part in to_iter {
    result.push(part.as_os_str());
  }

  result
}

#[cfg(test)]
mod test {
  use std::path::Path;

  use super::*;

  #[cfg(not(target_os = "windows"))]
  #[test]
  fn test_posix() {
    let test_cases = vec![
      ("/var/lib", "/var", ".."),
      ("/var/lib", "/bin", "../../bin"),
      ("/var/lib", "/var/lib", ""),
      ("/var/lib", "/var/apache", "../apache"),
      ("/var/", "/var/lib", "lib"),
      ("/", "/var/lib", "var/lib"),
      (
        "/foo/test",
        "/foo/test/bar/package.json",
        "bar/package.json",
      ),
      ("/Users/a/web/b/test/mails", "/Users/a/web/b", "../.."),
      ("/foo/bar/baz-quux", "/foo/bar/baz", "../baz"),
      ("/foo/bar/baz", "/foo/bar/baz-quux", "../baz-quux"),
      ("/baz-quux", "/baz", "../baz"),
      ("/baz", "/baz-quux", "../baz-quux"),
      ("/page1/page2/foo", "/", "../../.."),
    ];

    for (from, to, expected) in test_cases {
      let actual = relative(Path::new(from), Path::new(to))
        .to_string_lossy()
        .to_string();
      assert_eq!(actual, expected.to_string());
    }
  }

  #[cfg(target_os = "windows")]
  #[test]
  fn test_win32() {
    let test_cases = vec![
      ("c:/blah\\blah", "d:/games", "d:\\games"),
      ("c:/aaaa/bbbb", "c:/aaaa", ".."),
      ("c:/aaaa/bbbb", "c:/cccc", "..\\..\\cccc"),
      ("c:/aaaa/bbbb", "c:/aaaa/bbbb", ""),
      ("c:/aaaa/bbbb", "c:/aaaa/cccc", "..\\cccc"),
      ("c:/aaaa/", "c:/aaaa/cccc", "cccc"),
      ("c:/", "c:\\aaaa\\bbbb", "aaaa\\bbbb"),
      ("c:/aaaa/bbbb", "d:\\", "d:\\"),
      ("c:/AaAa/bbbb", "c:/aaaa/bbbb", ""),
      ("c:/aaaaa/", "c:/aaaa/cccc", "..\\aaaa\\cccc"),
      ("C:\\foo\\bar\\baz\\quux", "C:\\", "..\\..\\..\\.."),
      (
        "C:\\foo\\test",
        "C:\\foo\\test\\bar\\package.json",
        "bar\\package.json",
      ),
      ("C:\\foo\\bar\\baz-quux", "C:\\foo\\bar\\baz", "..\\baz"),
      (
        "C:\\foo\\bar\\baz",
        "C:\\foo\\bar\\baz-quux",
        "..\\baz-quux",
      ),
      ("\\\\foo\\bar", "\\\\foo\\bar\\baz", "baz"),
      ("\\\\foo\\bar\\baz", "\\\\foo\\bar", ".."),
      ("\\\\foo\\bar\\baz-quux", "\\\\foo\\bar\\baz", "..\\baz"),
      (
        "\\\\foo\\bar\\baz",
        "\\\\foo\\bar\\baz-quux",
        "..\\baz-quux",
      ),
      ("C:\\baz-quux", "C:\\baz", "..\\baz"),
      ("C:\\baz", "C:\\baz-quux", "..\\baz-quux"),
      ("\\\\foo\\baz-quux", "\\\\foo\\baz", "..\\baz"),
      ("\\\\foo\\baz", "\\\\foo\\baz-quux", "..\\baz-quux"),
      ("C:\\baz", "\\\\foo\\bar\\baz", "\\\\foo\\bar\\baz"),
      ("\\\\foo\\bar\\baz", "C:\\baz", "C:\\baz"),
    ];

    for (from, to, expected) in test_cases {
      let actual = relative(Path::new(from), Path::new(to))
        .to_string_lossy()
        .to_string();
      assert_eq!(actual, expected.to_string());
    }
  }
}
