use std::path::{Component, Components, Path, PathBuf};

fn normalize(parts: Components, allow_above_root: bool) -> Vec<Component> {
  let mut res = vec![];
  for p in parts {
    match &p {
      Component::CurDir => (),
      Component::ParentDir => {
        if !matches!(
          res.last(),
          Some(Component::ParentDir) | Some(Component::RootDir)
        ) {
          res.pop();
        } else if allow_above_root {
          res.push(p);
        }
      }
      _ => res.push(p),
    }
  }
  res
}

pub fn relative(from: &Path, to: &Path) -> PathBuf {
  if from == to {
    return PathBuf::new();
  }

  let is_from_absolute = matches!(from.components().next(), Some(Component::RootDir));
  let is_to_absolute = matches!(to.components().next(), Some(Component::RootDir));

  // At this point the path should be resolved to a full absolute path, but
  // handle relative paths to be safe

  // Normalize the path
  let from = normalize(from.components(), !is_from_absolute);
  let to = normalize(to.components(), !is_to_absolute);

  let mut from_iter = from.iter();
  let mut to_iter = to.iter();
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

  let to_iter = to.into_iter().skip(common_parts);
  for part in to_iter {
    result.push(part);
  }

  result
}

pub fn join(paths: &[&Path]) -> PathBuf {
  let paths = paths
    .iter()
    .filter(|path| path.components().last().is_some())
    .collect::<Vec<_>>();
  if paths.is_empty() {
    return PathBuf::from(".");
  }
  let mut buf = PathBuf::new();
  for (index, path) in paths.iter().enumerate() {
    for component in path.components() {
      match component {
        Component::RootDir => {
          if index == 0 {
            buf.push(component)
          }
        }
        Component::CurDir => (),
        Component::ParentDir => {
          if matches!(buf.components().last(), Some(Component::ParentDir) | None) {
            buf.push(component);
          } else {
            buf.pop();
          }
        }
        _ => {
          buf.push(component);
        }
      }
    }
  }
  if buf.components().last().is_none() {
    buf.push(Component::CurDir);
  }
  buf
}

#[cfg(test)]
mod test {
  use std::path::Path;

  use super::*;

  #[cfg(not(target_os = "windows"))]
  #[test]
  fn test_relative_posix() {
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
      // Fix https://github.com/web-infra-dev/rspack/issues/8219
      ("/", "/../maps/main.js.map", "maps/main.js.map"),
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
  fn test_relative_win32() {
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

  #[test]
  fn test_join() {
    let test_cases = vec![
      (vec![".", "x/b", "..", "/b/c.js"], "x/b/c.js"),
      (vec![], "."),
      (vec!["/.", "x/b", "..", "/b/c.js"], "/x/b/c.js"),
      (vec!["/foo", "../../../bar"], "/bar"),
      (vec!["foo", "../../../bar"], "../../bar"),
      (vec!["foo/", "../../../bar"], "../../bar"),
      (vec!["foo/x", "../../../bar"], "../bar"),
      (vec!["foo/x", "./bar"], "foo/x/bar"),
      (vec!["foo/x/", "./bar"], "foo/x/bar"),
      (vec!["foo/x/", ".", "bar"], "foo/x/bar"),
      // (vec!["./"], "./"),
      // (vec![".", "./"], "./"),
      (vec![".", ".", "."], "."),
      (vec![".", "./", "."], "."),
      (vec![".", "/./", "."], "."),
      (vec![".", "/////./", "."], "."),
      (vec!["."], "."),
      (vec!["", "."], "."),
      (vec!["", "foo"], "foo"),
      (vec!["foo", "/bar"], "foo/bar"),
      (vec!["", "/foo"], "/foo"),
      (vec!["", "", "/foo"], "/foo"),
      (vec!["", "", "foo"], "foo"),
      (vec!["foo", ""], "foo"),
      // (vec!["foo/", ""], "foo/"),
      (vec!["foo", "", "/bar"], "foo/bar"),
      (vec!["./", "..", "/foo"], "../foo"),
      (vec!["./", "..", "..", "/foo"], "../../foo"),
      (vec![".", "..", "..", "/foo"], "../../foo"),
      (vec!["", "..", "..", "/foo"], "../../foo"),
      (vec!["/"], "/"),
      (vec!["/", "."], "/"),
      (vec!["/", ".."], "/"),
      (vec!["/", "..", ".."], "/"),
      (vec![""], "."),
      (vec!["", ""], "."),
      (vec![" /foo"], " /foo"),
      (vec![" ", "foo"], " /foo"),
      (vec![" ", "."], " "),
      // (vec![" ", "/"], " /"),
      (vec![" ", ""], " "),
      (vec!["/", "foo"], "/foo"),
      (vec!["/", "/foo"], "/foo"),
      (vec!["/", "//foo"], "/foo"),
      (vec!["/", "", "/foo"], "/foo"),
      (vec!["", "/", "foo"], "/foo"),
      (vec!["", "/", "/foo"], "/foo"),
    ];

    for (paths, expected) in test_cases {
      let paths = paths.iter().map(Path::new).collect::<Vec<_>>();
      let actual = join(&paths).to_string_lossy().to_string();
      assert_eq!(actual, expected.to_string());
    }
  }
}
