use std::path::MAIN_SEPARATOR;

const MAIN_SEPARATOR_U8: u8 = MAIN_SEPARATOR as u8;

pub fn is_absolute_path(path: impl AsRef<str>) -> bool {
  let path = path.as_ref().as_bytes();

  #[cfg(not(windows))]
  {
    path.len() > 0 && path[0] == MAIN_SEPARATOR_U8
  }

  #[cfg(windows)]
  {
    path.len() > 2 && path[1] == b':' && path[2] == MAIN_SEPARATOR_U8
  }
}

/// Compute the relative path from `base` to `path`.
///
/// Performance-optimized using byte operations. Does not handle edge cases
/// like invalid paths or unusual Unicode. Both paths must be absolute.
pub fn to_relative_path(path: impl AsRef<str>, base: impl AsRef<str>) -> String {
  debug_assert!(is_absolute_path(&path));
  debug_assert!(is_absolute_path(&base));

  let path_str = path.as_ref();
  let base_str = base.as_ref();

  if path_str == base_str {
    return String::new();
  }

  let path_bytes = path_str.as_bytes();
  let base_bytes = base_str.as_bytes();

  // Find common prefix by iterating the shorter path
  let (short, long) = if path_bytes.len() < base_bytes.len() {
    (path_bytes, base_bytes)
  } else {
    (base_bytes, path_bytes)
  };
  let mut pointer = 0;
  let mut common_prefix_end = 0;
  for i in short {
    if i != &long[pointer] {
      break;
    }
    if i == &MAIN_SEPARATOR_U8 {
      common_prefix_end = pointer + 1;
    }
    pointer += 1;
  }
  // Handle case where one path is a prefix of the other (e.g., /a/b vs /a/b/c)
  if pointer == short.len() && short.len() != long.len() && long[pointer] == b'/' {
    common_prefix_end = pointer + 1;
  }

  // Extract the non-common parts, stripping trailing slashes
  let base_remainder = if common_prefix_end < base_str.len() {
    let last_index = base_bytes.len() - 1;
    if base_bytes[last_index] == MAIN_SEPARATOR_U8 {
      &base_str[common_prefix_end..last_index]
    } else {
      &base_str[common_prefix_end..]
    }
  } else {
    ""
  };
  let path_remainder = if common_prefix_end < path_str.len() {
    let last_index = path_bytes.len() - 1;
    if path_bytes[last_index] == MAIN_SEPARATOR_U8 {
      &path_str[common_prefix_end..last_index]
    } else {
      &path_str[common_prefix_end..]
    }
  } else {
    ""
  };

  // Build result: add ".." for each segment in base_remainder, then add path_remainder
  let mut result = String::new();

  if !base_remainder.is_empty() {
    result.push_str("..");
    for i in base_remainder.as_bytes() {
      if i == &MAIN_SEPARATOR_U8 {
        result.push_str("/..");
      }
    }
  }

  if !path_remainder.is_empty() {
    if !result.is_empty() {
      result.push('/');
    }
    for i in path_remainder.as_bytes() {
      if i == &MAIN_SEPARATOR_U8 {
        result.push('/');
      } else {
        result.push(char::from(*i));
      }
    }
  }

  result
}

/// Resolve a relative path against a base path to get an absolute path.
///
/// Performance-optimized using byte operations. Does not handle edge cases
/// like invalid paths. `base` must be absolute, `relative_path` must be relative.
/// Only handles Unix-like paths (Linux/macOS).
pub fn to_absolute_path(relative_path: impl AsRef<str>, base: impl AsRef<str>) -> String {
  debug_assert!(!is_absolute_path(&relative_path));
  debug_assert!(is_absolute_path(&base));

  let base_str = base.as_ref();
  let relative_str = relative_path.as_ref();

  let base_bytes = base_str.as_bytes();
  let relative_bytes = relative_str.as_bytes();

  // Count how many ".." are at the beginning of relative_path
  let mut dot_count = 0;
  let mut relative_level = 0;
  let mut relative_prefix_end = 0;
  let mut index = 0;
  loop {
    if index == relative_bytes.len() {
      // Check if the path ends with ".."
      if dot_count == 2 {
        relative_level += 1;
        relative_prefix_end = index;
      }
      // Check if the path ends with "."
      if dot_count == 1 {
        relative_prefix_end = index;
      }
      break;
    }
    let i = relative_bytes[index];
    if i == b'.' {
      dot_count += 1;
      index += 1;
      continue;
    }
    // relative path separator always /
    if i == b'/' {
      if dot_count == 2 {
        relative_level += 1;
        relative_prefix_end = index + 1;
      }
      dot_count = 0;
    } else {
      // Hit a non-dot, non-slash character, stop counting
      break;
    }
    index += 1;
  }

  // Find the base prefix by going back relative_level directories
  let mut base_prefix_end = base_bytes.len();
  if relative_level > 0 {
    let mut index = base_bytes.len() - 1;
    loop {
      if index == 0 {
        base_prefix_end = 0;
        break;
      }
      let b = base_bytes[index];
      if b == MAIN_SEPARATOR_U8 {
        relative_level -= 1;
        if relative_level == 0 {
          base_prefix_end = index;
          break;
        }
      }
      index -= 1;
    }
  }

  let mut result = String::new();
  if base_prefix_end != 0 {
    // not ends with /
    result.push_str(&base_str[0..base_prefix_end]);
  } else {
    #[cfg(not(windows))]
    result.push('/');

    #[cfg(windows)]
    result.push_str(&base_str[0..3])
  }

  // remove last separator
  let relative_remainder = if !relative_bytes.is_empty() {
    let last_index = relative_bytes.len() - 1;
    // relative path separator always /
    if relative_bytes[last_index] == b'/' && relative_prefix_end < last_index {
      &relative_str[relative_prefix_end..last_index]
    } else {
      &relative_str[relative_prefix_end..]
    }
  } else {
    ""
  };
  if !relative_remainder.is_empty() {
    // Only add separator if result doesn't already end with one
    if base_prefix_end != 0 {
      result.push(MAIN_SEPARATOR);
    }
    for i in relative_remainder.as_bytes() {
      if i == &b'/' {
        result.push(MAIN_SEPARATOR)
      } else {
        result.push(char::from(*i));
      }
    }
  }

  result
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use sugar_path::SugarPath;

  use super::{is_absolute_path, to_absolute_path, to_relative_path};

  #[test]
  fn should_is_absolute_path_work() {
    let cases = vec!["/a/b/c", "/a/b/c/", "a/b/c", "./a/b/c", "../a/b/c"];

    for path in cases {
      assert_eq!(
        is_absolute_path(path),
        PathBuf::from(path).is_absolute(),
        "Failed: path is {:?}",
        path,
      );
    }
  }

  #[test]
  fn should_to_relative_path_work() {
    let cases = vec![
      ("/a/b/c", "/a/b"),
      ("/a/b/c", "/a/b/"),
      ("/a/b/c/", "/a/b"),
      ("/a/b/c/", "/a/b/"),
      ("/a/b/c/", "/a/b/c"),
      ("/a/b/c/", "/a/b/c/"),
      ("/a/b/c", "/a/b/c/"),
      ("/a/b/c", "/a/b/c"),
      ("/a/b", "/a/b/c"),
      ("/a/b/", "/a/b/c"),
      ("/a/b", "/a/b/c/"),
      ("/a/b/", "/a/b/c/"),
    ];

    for (path, base) in cases {
      assert_eq!(
        to_relative_path(path, base),
        path.relative(base).to_string_lossy(),
        "Failed: path is {:?}, base is {:?}",
        path,
        base
      );
    }
  }

  #[test]
  #[cfg(not(windows))]
  fn should_to_absolute_path_work() {
    let test_cases = vec![
      ("src/main.rs", "/home/user/project"),
      ("../other/file.txt", "/home/user/project"),
      ("../../etc/config.ini", "/home/user/project"),
      ("c/d", "/a/b"),
      ("..", "/a/b"),
      ("../..", "/a/b/c"),
      ("", "/a"),
      (".", "/a/b"),
      ("../c/", "/a"),
      ("../../../", "/a"),
    ];

    for (relative, base) in test_cases {
      let result = to_absolute_path(relative, base);
      let expected = format!("{}/{}", base, relative).normalize();
      assert_eq!(
        result,
        expected.to_string_lossy(),
        "Failed: base={:?}, relative={:?}",
        base,
        relative
      );
    }
  }
}
