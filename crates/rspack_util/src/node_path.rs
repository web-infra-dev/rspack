use std::borrow::Cow;

use rspack_paths::{Utf8Path, Utf8PathBuf};

fn is_path_separator(byte: &u8) -> bool {
  *byte == b'/' || *byte == b'\\'
}

fn is_posix_path_separator(byte: &u8) -> bool {
  *byte == b'/'
}

fn is_windows_device_root(byte: &u8) -> bool {
  (*byte >= b'A' && *byte <= b'Z') || (*byte >= b'a' && *byte <= b'z')
}

// Resolves . and .. elements in a path with directory names
fn normalize_string(
  path: impl AsRef<[u8]>,
  allow_above_root: bool,
  separator: u8,
  is_path_separator: &dyn Fn(&u8) -> bool,
) -> String {
  let path = path.as_ref();

  let mut res: Vec<u8> = Vec::with_capacity(path.len());
  let mut last_segment_length = 0;
  let mut last_slash = -1;
  let mut dots = 0;
  let mut code = &b'\0';

  for (i, ch) in path.iter().enumerate().chain(Some((path.len(), &b'\0'))) {
    if i < path.len() {
      code = ch;
    } else if is_path_separator(code) {
      break;
    } else {
      code = &b'/';
    }

    if is_path_separator(code) {
      if last_slash == i as isize - 1 || dots == 1 {
        // NOOP
      } else if dots == 2 {
        if res.len() < 2 || last_segment_length != 2 || &res[res.len() - 2..] != b".." {
          if res.len() > 2 {
            if let Some(last_slash_index) = res.iter().rposition(|b| *b == separator) {
              res.truncate(last_slash_index);
              last_segment_length = res.len()
                - res
                  .iter()
                  .rposition(|b| *b == separator)
                  .map_or(0, |i| i + 1);
            } else {
              res.clear();
              last_segment_length = 0;
            }
            last_slash = i as isize;
            dots = 0;
            continue;
          } else if !res.is_empty() {
            res.clear();
            last_segment_length = 0;
            last_slash = i as isize;
            dots = 0;
            continue;
          }
        }
        if allow_above_root {
          if !res.is_empty() {
            res.push(separator);
          }
          res.extend(b"..");
          last_segment_length = 2;
        }
      } else {
        if !res.is_empty() {
          res.push(separator);
          res.extend(&path[(last_slash + 1) as usize..i]);
        } else {
          res.extend(&path[(last_slash + 1) as usize..i]);
        }
        last_segment_length = (i as isize - last_slash - 1) as usize;
      }
      last_slash = i as isize;
      dots = 0;
    } else if *code == b'.' && dots != -1 {
      dots += 1;
    } else {
      dots = -1;
    }
  }

  unsafe { String::from_utf8_unchecked(res) }
}

pub trait NodePath {
  fn node_join(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf;

  fn node_join_posix(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf;

  fn node_join_win32(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf;

  fn node_push(&mut self, path: impl AsRef<Utf8Path>);

  fn node_push_posix(&mut self, path: impl AsRef<Utf8Path>);

  fn node_push_win32(&mut self, path: impl AsRef<Utf8Path>);

  #[must_use]
  fn node_normalize(&self) -> Utf8PathBuf;

  fn node_normalize_posix(&self) -> Utf8PathBuf;

  fn node_normalize_win32(&self) -> Utf8PathBuf;

  fn node_is_absolute(&self) -> bool;

  fn node_is_absolute_posix(&self) -> bool;

  fn node_is_absolute_win32(&self) -> bool;
}

impl NodePath for Utf8Path {
  #[inline]
  fn node_join(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
    self.to_path_buf().node_join(path.as_ref())
  }

  #[inline]
  fn node_join_posix(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
    self.to_path_buf().node_join_posix(path.as_ref())
  }

  fn node_join_win32(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
    self.to_path_buf().node_join_win32(path.as_ref())
  }

  #[inline]
  fn node_push(&mut self, path: impl AsRef<Utf8Path>) {
    self.to_path_buf().node_push(path.as_ref());
  }

  #[inline]
  fn node_push_posix(&mut self, path: impl AsRef<Utf8Path>) {
    self.to_path_buf().node_push_posix(path.as_ref());
  }

  #[inline]
  fn node_push_win32(&mut self, path: impl AsRef<Utf8Path>) {
    self.to_path_buf().node_push_win32(path.as_ref());
  }

  #[inline]
  fn node_normalize(&self) -> Utf8PathBuf {
    self.to_path_buf().node_normalize()
  }

  #[inline]
  fn node_normalize_posix(&self) -> Utf8PathBuf {
    self.to_path_buf().node_normalize_posix()
  }

  #[inline]
  fn node_normalize_win32(&self) -> Utf8PathBuf {
    self.to_path_buf().node_normalize_win32()
  }

  #[inline]
  fn node_is_absolute(&self) -> bool {
    self.to_path_buf().node_is_absolute()
  }

  #[inline]
  fn node_is_absolute_posix(&self) -> bool {
    self.to_path_buf().node_is_absolute_posix()
  }

  #[inline]
  fn node_is_absolute_win32(&self) -> bool {
    self.to_path_buf().node_is_absolute_win32()
  }
}

impl NodePath for Utf8PathBuf {
  #[inline]
  fn node_join(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
    if cfg!(windows) {
      self.node_join_win32(path)
    } else {
      self.node_join_posix(path)
    }
  }

  fn node_join_posix(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
    let mut buf = self.to_path_buf();
    buf.node_push_posix(path);
    buf
  }

  fn node_join_win32(&self, path: impl AsRef<Utf8Path>) -> Utf8PathBuf {
    let mut buf = self.to_path_buf();
    buf.node_push_win32(path);
    buf
  }

  #[inline]
  fn node_push(&mut self, path: impl AsRef<Utf8Path>) {
    if cfg!(windows) {
      self.node_push_win32(path);
    } else {
      self.node_push_posix(path);
    }
  }

  fn node_push_posix(&mut self, path: impl AsRef<Utf8Path>) {
    let path = path.as_ref().as_str();
    if path.is_empty() {
      return;
    }

    if self.as_str().is_empty() {
      self.push(path);
      return;
    }

    // in general, a separator is needed if the rightmost byte is not a separator
    let buf = self.as_os_str().as_encoded_bytes();
    let need_sep = buf.last().is_some_and(|c| !is_posix_path_separator(c))
      && path
        .as_bytes()
        .first()
        .is_some_and(|c| !is_posix_path_separator(c));

    let mut string = self.as_str().to_string();
    if need_sep {
      string.push('/');
    }
    string.push_str(path);
    *self = Utf8PathBuf::from(string);
  }

  fn node_push_win32(&mut self, path: impl AsRef<Utf8Path>) {
    let path = path.as_ref().as_str();
    if path.is_empty() {
      return;
    }

    if self.as_str().is_empty() {
      self.push(path);
      return;
    }

    // in general, a separator is needed if the rightmost byte is not a separator
    let buf = self.as_os_str().as_encoded_bytes();
    let need_sep = buf.last().is_some_and(|c| !is_path_separator(c))
      && path
        .as_bytes()
        .first()
        .is_some_and(|c| !is_path_separator(c));

    let mut joined = self.to_string();
    if need_sep {
      joined.push('\\');
    }
    joined.push_str(path);

    let first_part = self.as_str().as_bytes();
    // Make sure that the joined path doesn't start with two slashes, because
    // normalize() will mistake it for a UNC path then.
    //
    // This step is skipped when it is very clear that the user actually
    // intended to point at a UNC path. This is assumed when the first
    // non-empty string arguments starts with exactly two slashes followed by
    // at least one more non-slash character.
    //
    // Note that for normalize() to treat a path as a UNC path it needs to
    // have at least 2 components, so we don't filter for that here.
    // This means that the user can use join to construct UNC paths from
    // a server name and a share name; for example:
    //   path.join('//server', 'share') -> '\\\\server\\share\\')
    let mut needs_replace = true;
    let mut slash_count = 0;

    if is_path_separator(&first_part[0]) {
      slash_count += 1;
      if first_part.len() > 1 && is_path_separator(&first_part[1]) {
        slash_count += 1;
        if first_part.len() > 2 {
          if is_path_separator(&first_part[2]) {
            slash_count += 1;
          } else {
            // We matched a UNC path in the first part
            needs_replace = false;
          }
        }
      }
    }

    if needs_replace {
      // Find any more consecutive slashes we need to replace
      while slash_count < joined.len()
        && joined
          .as_bytes()
          .get(slash_count)
          .is_some_and(is_path_separator)
      {
        slash_count += 1;
      }

      // Replace the slashes if needed
      if slash_count >= 2 {
        *self = Utf8PathBuf::from(format!("\\{}", &joined[slash_count..]));
        return;
      }
    }

    *self = Utf8PathBuf::from(joined);
  }

  #[inline]
  fn node_normalize(&self) -> Utf8PathBuf {
    if cfg!(windows) {
      self.node_normalize_win32()
    } else {
      self.node_normalize_posix()
    }
  }

  fn node_normalize_posix(&self) -> Utf8PathBuf {
    let path = self.as_str().as_bytes();

    if path.is_empty() {
      return Utf8PathBuf::from(".");
    }

    let is_absolute = path.first() == Some(&b'/');
    let trailing_separator = path.last() == Some(&b'/');

    // Normalize the path
    let mut normalized_path = normalize_string(path, !is_absolute, b'/', &is_posix_path_separator);

    if normalized_path.is_empty() {
      if is_absolute {
        return Utf8PathBuf::from("/");
      }
      return if trailing_separator {
        Utf8PathBuf::from("./")
      } else {
        Utf8PathBuf::from(".")
      };
    }

    if trailing_separator {
      normalized_path.push('/');
    }

    if is_absolute {
      Utf8PathBuf::from(format!("/{normalized_path}"))
    } else {
      Utf8PathBuf::from(normalized_path)
    }
  }

  fn node_normalize_win32(&self) -> Utf8PathBuf {
    let path = self.as_str();
    if path.is_empty() {
      return Utf8PathBuf::from(".");
    }

    let len = path.len();

    let mut root_end = 0;
    let mut device = None;
    let mut is_absolute = false;
    let code = &path.as_bytes()[0];

    // Try to match a root
    if len == 1 {
      // `path` contains just a single char, exit early to avoid
      // unnecessary work
      if is_posix_path_separator(code) {
        return Utf8PathBuf::from("\\");
      }
    }
    if is_path_separator(code) {
      // Possible UNC root

      // If we started with a separator, we know we at least have an absolute
      // path of some kind (UNC or otherwise)
      is_absolute = true;

      if path.as_bytes().get(1).is_some_and(is_path_separator) {
        // Matched double path separator at beginning
        let mut j = 2;
        let mut last = j;
        // Match 1 or more non-path separators
        while j < len && !is_path_separator(&path.as_bytes()[j]) {
          j += 1;
        }
        if j < len && j != last {
          let first_part = &path[last..j];
          // Matched!
          last = j;
          // Match 1 or more path separators
          while j < len && is_path_separator(&path.as_bytes()[j]) {
            j += 1;
          }
          if j < len && j != last {
            // Matched!
            last = j;
            // Match 1 or more non-path separators
            while j < len && !is_path_separator(&path.as_bytes()[j]) {
              j += 1;
            }
            if j < len || j != last {
              if first_part == "." || first_part == "?" {
                // We matched a device root (e.g. \\\\.\\PHYSICALDRIVE0)
                device = Some(Cow::from(format!("\\\\{first_part}")));
                root_end = 4;
              } else if j == len {
                // We matched a UNC root only
                // Return the normalized version of the UNC root since there
                // is nothing left to process
                return Utf8PathBuf::from(format!("\\\\{}\\{}\\", first_part, &path[last..]));
              } else {
                // We matched a UNC root with leftovers
                device = Some(Cow::from(format!("\\\\{}\\{}", first_part, &path[last..j])));
                root_end = j;
              }
            }
          }
        }
      } else {
        root_end = 1;
      }
    } else if is_windows_device_root(&path.as_bytes()[0]) && path.as_bytes().get(1) == Some(&b':') {
      // Possible device root
      device = Some(Cow::from(&path[0..2]));
      root_end = 2;
      if path.len() > 2 && path.as_bytes().get(2).is_some_and(is_path_separator) {
        // Treat separator following drive name as an absolute path indicator
        is_absolute = true;
        root_end = 3;
      }
    }

    let mut tail = if root_end < path.len() {
      normalize_string(&path[root_end..], !is_absolute, b'\\', &is_path_separator)
    } else {
      String::new()
    };

    if tail.is_empty() && !is_absolute {
      tail = ".".to_string();
    }

    if !tail.is_empty() && path.as_bytes().last().is_some_and(is_path_separator) {
      tail.push('\\');
    }

    if !is_absolute && device.is_none() && path.contains(':') {
      // If the original path was not absolute and if we have not been able to
      // resolve it relative to a particular device, we need to ensure that the
      // `tail` has not become something that Windows might interpret as an
      // absolute path. See CVE-2024-36139.
      if tail.len() >= 2
        && is_windows_device_root(&tail.as_bytes()[0])
        && tail.as_bytes().get(1) == Some(&b':')
      {
        return Utf8PathBuf::from(format!(".\\{tail}"));
      }
      let mut index = path.find(':');
      while let Some(i) = index {
        if i == path.len() - 1 || path.as_bytes().get(i + 1).is_some_and(is_path_separator) {
          return Utf8PathBuf::from(format!(".\\{tail}"));
        }
        index = path[i + 1..].find(':').map(|next| next + i + 1);
      }
    }

    match device {
      Some(device) if is_absolute => Utf8PathBuf::from(format!("{device}\\{tail}")),
      Some(device) => Utf8PathBuf::from(format!("{device}{tail}")),
      None if is_absolute => Utf8PathBuf::from(format!("\\{tail}")),
      _ => Utf8PathBuf::from(tail),
    }
  }

  #[inline]
  fn node_is_absolute(&self) -> bool {
    if cfg!(windows) {
      self.node_is_absolute_win32()
    } else {
      self.node_is_absolute_posix()
    }
  }

  fn node_is_absolute_posix(&self) -> bool {
    // POSIX absolute path starts with '/'
    self.as_str().starts_with('/')
  }

  fn node_is_absolute_win32(&self) -> bool {
    let path = self.as_str();

    if path.is_empty() {
      return false;
    }

    let bytes = path.as_bytes();

    // Mirrors Node.js `path.win32.isAbsolute`: first separator is absolute.
    if is_path_separator(&bytes[0]) {
      return true;
    }

    if path.len() > 2
      && is_windows_device_root(&bytes[0])
      && bytes[1] == b':'
      && is_path_separator(&bytes[2])
    {
      return true;
    }

    false
  }
}

#[cfg(test)]
mod test {

  use cow_utils::CowUtils;

  use super::*;

  // Test cases from https://github.com/nodejs/node/blob/1b2d2f7e682268228b1352cba7389db01614812a/test/parallel/test-path-normalize.js

  #[test]
  fn test_path_normalize_windows() {
    assert_eq!(
      Utf8Path::new("./fixtures///b/../b/c.js")
        .node_normalize_win32()
        .as_str(),
      "fixtures\\b\\c.js"
    );
    assert_eq!(
      Utf8Path::new("/foo/../../../bar")
        .node_normalize_win32()
        .as_str(),
      "\\bar"
    );
    assert_eq!(
      Utf8Path::new("a//b//../b").node_normalize_win32().as_str(),
      "a\\b"
    );
    assert_eq!(
      Utf8Path::new("a//b//./c").node_normalize_win32().as_str(),
      "a\\b\\c"
    );
    assert_eq!(
      Utf8Path::new("a//b//.").node_normalize_win32().as_str(),
      "a\\b"
    );
    assert_eq!(
      Utf8Path::new("//server/share/dir/file.ext")
        .node_normalize_win32()
        .as_str(),
      "\\\\server\\share\\dir\\file.ext"
    );
    assert_eq!(
      Utf8Path::new("/a/b/c/../../../x/y/z")
        .node_normalize_win32()
        .as_str(),
      "\\x\\y\\z"
    );
    assert_eq!(Utf8Path::new("C:").node_normalize_win32().as_str(), "C:.");
    assert_eq!(
      Utf8Path::new("C:..\\abc").node_normalize_win32().as_str(),
      "C:..\\abc"
    );
    assert_eq!(
      Utf8Path::new("C:..\\..\\abc\\..\\def")
        .node_normalize_win32()
        .as_str(),
      "C:..\\..\\def"
    );
    assert_eq!(
      Utf8Path::new("C:\\.").node_normalize_win32().as_str(),
      "C:\\"
    );
    assert_eq!(
      Utf8Path::new("file:stream").node_normalize_win32().as_str(),
      "file:stream"
    );
    assert_eq!(
      Utf8Path::new("bar\\foo..\\..\\")
        .node_normalize_win32()
        .as_str(),
      "bar\\"
    );
    assert_eq!(
      Utf8Path::new("bar\\foo..\\..")
        .node_normalize_win32()
        .as_str(),
      "bar"
    );
    assert_eq!(
      Utf8Path::new("bar\\foo..\\..\\baz")
        .node_normalize_win32()
        .as_str(),
      "bar\\baz"
    );
    assert_eq!(
      Utf8Path::new("bar\\foo..\\")
        .node_normalize_win32()
        .as_str(),
      "bar\\foo..\\"
    );
    assert_eq!(
      Utf8Path::new("bar\\foo..").node_normalize_win32().as_str(),
      "bar\\foo.."
    );
    assert_eq!(
      Utf8Path::new("..\\foo..\\..\\..\\bar")
        .node_normalize_win32()
        .as_str(),
      "..\\..\\bar"
    );
    assert_eq!(
      Utf8Path::new("..\\...\\..\\.\\...\\..\\..\\bar")
        .node_normalize_win32()
        .as_str(),
      "..\\..\\bar"
    );
    assert_eq!(
      Utf8Path::new("../../../foo/../../../bar")
        .node_normalize_win32()
        .as_str(),
      "..\\..\\..\\..\\..\\bar"
    );
    assert_eq!(
      Utf8Path::new("../../../foo/../../../bar/../../")
        .node_normalize_win32()
        .as_str(),
      "..\\..\\..\\..\\..\\..\\"
    );
    assert_eq!(
      Utf8Path::new("../foobar/barfoo/foo/../../../bar/../../")
        .node_normalize_win32()
        .as_str(),
      "..\\..\\"
    );
    assert_eq!(
      Utf8Path::new("../.../../foobar/../../../bar/../../baz")
        .node_normalize_win32()
        .as_str(),
      "..\\..\\..\\..\\baz"
    );
    assert_eq!(
      Utf8Path::new("foo/bar\\baz")
        .node_normalize_win32()
        .as_str(),
      "foo\\bar\\baz"
    );
    assert_eq!(
      Utf8Path::new("\\\\.\\foo").node_normalize_win32().as_str(),
      "\\\\.\\foo"
    );
    assert_eq!(
      Utf8Path::new("\\\\.\\foo\\")
        .node_normalize_win32()
        .as_str(),
      "\\\\.\\foo\\"
    );
    assert_eq!(
      Utf8Path::new("test/../C:/Windows")
        .node_normalize_win32()
        .as_str(),
      ".\\C:\\Windows"
    );
    assert_eq!(
      Utf8Path::new("test/../C:Windows")
        .node_normalize_win32()
        .as_str(),
      ".\\C:Windows"
    );
    assert_eq!(
      Utf8Path::new("./upload/../C:/Windows")
        .node_normalize_win32()
        .as_str(),
      ".\\C:\\Windows"
    );
    assert_eq!(
      Utf8Path::new("./upload/../C:x")
        .node_normalize_win32()
        .as_str(),
      ".\\C:x"
    );
    assert_eq!(
      Utf8Path::new("test/../??/D:/Test")
        .node_normalize_win32()
        .as_str(),
      ".\\??\\D:\\Test"
    );
    assert_eq!(
      Utf8Path::new("test/C:/../../F:")
        .node_normalize_win32()
        .as_str(),
      ".\\F:"
    );
    assert_eq!(
      Utf8Path::new("test/C:foo/../../F:")
        .node_normalize_win32()
        .as_str(),
      ".\\F:"
    );
    assert_eq!(
      Utf8Path::new("test/C:/../../F:\\")
        .node_normalize_win32()
        .as_str(),
      ".\\F:\\"
    );
    assert_eq!(
      Utf8Path::new("test/C:foo/../../F:\\")
        .node_normalize_win32()
        .as_str(),
      ".\\F:\\"
    );
    assert_eq!(
      Utf8Path::new("test/C:/../../F:x")
        .node_normalize_win32()
        .as_str(),
      ".\\F:x"
    );
    assert_eq!(
      Utf8Path::new("test/C:foo/../../F:x")
        .node_normalize_win32()
        .as_str(),
      ".\\F:x"
    );
    assert_eq!(
      Utf8Path::new("/test/../??/D:/Test")
        .node_normalize_win32()
        .as_str(),
      "\\??\\D:\\Test"
    );
    assert_eq!(
      Utf8Path::new("/test/../?/D:/Test")
        .node_normalize_win32()
        .as_str(),
      "\\?\\D:\\Test"
    );
    assert_eq!(
      Utf8Path::new("//test/../??/D:/Test")
        .node_normalize_win32()
        .as_str(),
      "\\\\test\\..\\??\\D:\\Test"
    );
    assert_eq!(
      Utf8Path::new("//test/../?/D:/Test")
        .node_normalize_win32()
        .as_str(),
      "\\\\test\\..\\?\\D:\\Test"
    );
    assert_eq!(
      Utf8Path::new("\\\\?\\test/../?/D:/Test")
        .node_normalize_win32()
        .as_str(),
      "\\\\?\\?\\D:\\Test"
    );
    assert_eq!(
      Utf8Path::new("\\\\?\\test/../../?/D:/Test")
        .node_normalize_win32()
        .as_str(),
      "\\\\?\\?\\D:\\Test"
    );
    assert_eq!(
      Utf8Path::new("\\\\.\\test/../?/D:/Test")
        .node_normalize_win32()
        .as_str(),
      "\\\\.\\?\\D:\\Test"
    );
    assert_eq!(
      Utf8Path::new("\\\\.\\test/../../?/D:/Test")
        .node_normalize_win32()
        .as_str(),
      "\\\\.\\?\\D:\\Test"
    );
    assert_eq!(
      Utf8Path::new("//server/share/dir/../../../?/D:/file")
        .node_normalize_win32()
        .as_str(),
      "\\\\server\\share\\?\\D:\\file"
    );
    assert_eq!(
      Utf8Path::new("//server/goodshare/../badshare/file")
        .node_normalize_win32()
        .as_str(),
      "\\\\server\\goodshare\\badshare\\file"
    );
  }

  #[test]
  fn test_path_normalize_posix() {
    assert_eq!(
      Utf8Path::new("./fixtures///b/../b/c.js")
        .node_normalize_posix()
        .as_str(),
      "fixtures/b/c.js"
    );
    assert_eq!(
      Utf8Path::new("/foo/../../../bar")
        .node_normalize_posix()
        .as_str(),
      "/bar"
    );
    assert_eq!(
      Utf8Path::new("a//b//../b").node_normalize_posix().as_str(),
      "a/b"
    );
    assert_eq!(
      Utf8Path::new("a//b//./c").node_normalize_posix().as_str(),
      "a/b/c"
    );
    assert_eq!(
      Utf8Path::new("./fixtures///b/../b/c.js")
        .node_normalize_posix()
        .as_str(),
      "fixtures/b/c.js"
    );
    assert_eq!(
      Utf8Path::new("a//b//.").node_normalize_posix().as_str(),
      "a/b"
    );
    assert_eq!(
      Utf8Path::new("/a/b/c/../../../x/y/z")
        .node_normalize_posix()
        .as_str(),
      "/x/y/z"
    );
    assert_eq!(
      Utf8Path::new("///..//./foo/.//bar")
        .node_normalize_posix()
        .as_str(),
      "/foo/bar"
    );
    assert_eq!(
      Utf8Path::new("bar/foo../../")
        .node_normalize_posix()
        .as_str(),
      "bar/"
    );
    assert_eq!(
      Utf8Path::new("bar/foo../..")
        .node_normalize_posix()
        .as_str(),
      "bar"
    );
    assert_eq!(
      Utf8Path::new("bar/foo../../baz")
        .node_normalize_posix()
        .as_str(),
      "bar/baz"
    );
    assert_eq!(
      Utf8Path::new("bar/foo../").node_normalize_posix().as_str(),
      "bar/foo../"
    );
    assert_eq!(
      Utf8Path::new("bar/foo..").node_normalize_posix().as_str(),
      "bar/foo.."
    );
    assert_eq!(
      Utf8Path::new("../foo../../../bar")
        .node_normalize_posix()
        .as_str(),
      "../../bar"
    );
    assert_eq!(
      Utf8Path::new("../.../.././.../../../bar")
        .node_normalize_posix()
        .as_str(),
      "../../bar"
    );
    assert_eq!(
      Utf8Path::new("../../../foo/../../../bar")
        .node_normalize_posix()
        .as_str(),
      "../../../../../bar"
    );
    assert_eq!(
      Utf8Path::new("../../../foo/../../../bar/../../")
        .node_normalize_posix()
        .as_str(),
      "../../../../../../"
    );
    assert_eq!(
      Utf8Path::new("../foobar/barfoo/foo/../../../bar/../../")
        .node_normalize_posix()
        .as_str(),
      "../../"
    );
    assert_eq!(
      Utf8Path::new("../.../../foobar/../../../bar/../../baz")
        .node_normalize_posix()
        .as_str(),
      "../../../../baz"
    );
    assert_eq!(
      Utf8Path::new("foo/bar\\baz")
        .node_normalize_posix()
        .as_str(),
      "foo/bar\\baz"
    );
  }

  const JOIN_TESTS: &[(&[&str], &str)] = &[
    (&[".", "x/b", "..", "/b/c.js"], "x/b/c.js"),
    (&[], "."),
    (&["/.", "x/b", "..", "/b/c.js"], "/x/b/c.js"),
    (&["/foo", "../../../bar"], "/bar"),
    (&["foo", "../../../bar"], "../../bar"),
    (&["foo/", "../../../bar"], "../../bar"),
    (&["foo/x", "../../../bar"], "../bar"),
    (&["foo/x", "./bar"], "foo/x/bar"),
    (&["foo/x/", "./bar"], "foo/x/bar"),
    (&["foo/x/", ".", "bar"], "foo/x/bar"),
    (&["./"], "./"),
    (&[".", "./"], "./"),
    (&[".", ".", "."], "."),
    (&[".", "./", "."], "."),
    (&[".", "/./", "."], "."),
    (&[".", "/////./", "."], "."),
    (&["."], "."),
    (&["", "."], "."),
    (&["", "foo"], "foo"),
    (&["foo", "/bar"], "foo/bar"),
    (&["", "/foo"], "/foo"),
    (&["", "", "/foo"], "/foo"),
    (&["", "", "foo"], "foo"),
    (&["foo", ""], "foo"),
    (&["foo/", ""], "foo/"),
    (&["foo", "", "/bar"], "foo/bar"),
    (&["./", "..", "/foo"], "../foo"),
    (&["./", "..", "..", "/foo"], "../../foo"),
    (&[".", "..", "..", "/foo"], "../../foo"),
    (&["", "..", "..", "/foo"], "../../foo"),
    (&["/"], "/"),
    (&["/", "."], "/"),
    (&["/", ".."], "/"),
    (&["/", "..", ".."], "/"),
    (&[""], "."),
    (&["", ""], "."),
    (&[" /foo"], " /foo"),
    (&[" ", "foo"], " /foo"),
    (&[" ", "."], " "),
    (&[" ", "/"], " /"),
    (&[" ", ""], " "),
    (&["/", "foo"], "/foo"),
    (&["/", "/foo"], "/foo"),
    (&["/", "//foo"], "/foo"),
    (&["/", "", "/foo"], "/foo"),
    (&["", "/", "foo"], "/foo"),
    (&["", "/", "/foo"], "/foo"),
  ];

  #[test]
  fn test_node_join() {
    JOIN_TESTS.iter().for_each(|(paths, expected)| {
      let mut path = Utf8PathBuf::new();
      for p in *paths {
        path = path.node_join_posix(p);
      }
      assert_eq!(path.node_normalize_posix().as_str(), *expected);

      let mut path = Utf8PathBuf::new();
      for p in *paths {
        path = path.node_join_win32(p);
      }
      // For non-Windows specific tests with the Windows join(), we need to try
      // replacing the slashes since the non-Windows specific tests" `expected`
      // use forward slashes
      assert_eq!(
        path
          .node_normalize_win32()
          .as_str()
          .cow_replace("\\", "/")
          .as_ref(),
        *expected
      );
    });
  }

  const WINDOWS_SPECIFIC_JOIN_TESTS: &[(&[&str], &str)] = &[
    // UNC path expected
    (&["//foo/bar"], "\\\\foo\\bar\\"),
    (&["\\/foo/bar"], "\\\\foo\\bar\\"),
    (&["\\\\foo/bar"], "\\\\foo\\bar\\"),
    // UNC path expected - server and share separate
    (&["//foo", "bar"], "\\\\foo\\bar\\"),
    (&["//foo/", "bar"], "\\\\foo\\bar\\"),
    (&["//foo", "/bar"], "\\\\foo\\bar\\"),
    // UNC path expected - questionable
    (&["//foo", "", "bar"], "\\\\foo\\bar\\"),
    (&["//foo/", "", "bar"], "\\\\foo\\bar\\"),
    (&["//foo/", "", "/bar"], "\\\\foo\\bar\\"),
    // UNC path expected - even more questionable
    (&["", "//foo", "bar"], "\\\\foo\\bar\\"),
    (&["", "//foo/", "bar"], "\\\\foo\\bar\\"),
    (&["", "//foo/", "/bar"], "\\\\foo\\bar\\"),
    // No UNC path expected (no double slash in first component)
    (&["\\", "foo/bar"], "\\foo\\bar"),
    (&["\\", "/foo/bar"], "\\foo\\bar"),
    (&["", "/", "/foo/bar"], "\\foo\\bar"),
    // No UNC path expected (no non-slashes in first component -
    // questionable)
    (&["//", "foo/bar"], "\\foo\\bar"),
    (&["//", "/foo/bar"], "\\foo\\bar"),
    (&["\\\\", "/", "/foo/bar"], "\\foo\\bar"),
    (&["//"], "\\"),
    // No UNC path expected (share name missing - questionable).
    (&["//foo"], "\\foo"),
    (&["//foo/"], "\\foo\\"),
    (&["//foo", "/"], "\\foo\\"),
    (&["//foo", "", "/"], "\\foo\\"),
    // No UNC path expected (too many leading slashes - questionable)
    (&["///foo/bar"], "\\foo\\bar"),
    (&["////foo", "bar"], "\\foo\\bar"),
    (&["\\\\\\/foo/bar"], "\\foo\\bar"),
    // Drive-relative vs drive-absolute paths. This merely describes the
    // status quo, rather than being obviously right
    (&["c:"], "c:."),
    (&["c:."], "c:."),
    (&["c:", ""], "c:."),
    (&["", "c:"], "c:."),
    (&["c:.", "/"], "c:.\\"),
    (&["c:.", "file"], "c:file"),
    (&["c:", "/"], "c:\\"),
    (&["c:", "file"], "c:\\file"),
    // Path traversal in previous versions of Node.js.
    (&["./upload", "/../C:/Windows"], ".\\C:\\Windows"),
    (&["upload", "../", "C:foo"], ".\\C:foo"),
    (&["test/..", "??/D:/Test"], ".\\??\\D:\\Test"),
    (&["test", "..", "D:"], ".\\D:"),
    (&["test", "..", "D:\\"], ".\\D:\\"),
    (&["test", "..", "D:foo"], ".\\D:foo"),
  ];

  #[test]
  fn test_path_join_windows() {
    WINDOWS_SPECIFIC_JOIN_TESTS
      .iter()
      .for_each(|(paths, expected)| {
        let mut path = Utf8PathBuf::new();
        for p in *paths {
          path = path.node_join_win32(p);
        }
        assert_eq!(path.node_normalize_win32().as_str(), *expected);
      });
  }

  // Test cases from https://github.com/nodejs/node/blob/5cf3c3e24c7257a0c6192ed8ef71efec8ddac22b/test/parallel/test-path-isabsolute.js
  #[test]
  fn test_node_is_absolute() {
    // win32 cases
    assert!(Utf8Path::new("/").node_is_absolute_win32());
    assert!(Utf8Path::new("//").node_is_absolute_win32());
    assert!(Utf8Path::new("//server").node_is_absolute_win32());
    assert!(Utf8Path::new("//server/file").node_is_absolute_win32());
    assert!(Utf8Path::new("\\\\server\\file").node_is_absolute_win32());
    assert!(Utf8Path::new("\\\\server").node_is_absolute_win32());
    assert!(Utf8Path::new("\\\\").node_is_absolute_win32());
    assert!(!Utf8Path::new("c").node_is_absolute_win32());
    assert!(!Utf8Path::new("c:").node_is_absolute_win32());
    assert!(Utf8Path::new("c:\\").node_is_absolute_win32());
    assert!(Utf8Path::new("c:/").node_is_absolute_win32());
    assert!(Utf8Path::new("c://").node_is_absolute_win32());
    assert!(Utf8Path::new("C:/Users/").node_is_absolute_win32());
    assert!(Utf8Path::new("C:\\Users\\").node_is_absolute_win32());
    assert!(!Utf8Path::new("C:cwd/another").node_is_absolute_win32());
    assert!(!Utf8Path::new("C:cwd\\another").node_is_absolute_win32());
    assert!(!Utf8Path::new("directory/directory").node_is_absolute_win32());
    assert!(!Utf8Path::new("directory\\directory").node_is_absolute_win32());

    // posix cases
    assert!(Utf8Path::new("/home/foo").node_is_absolute_posix());
    assert!(Utf8Path::new("/home/foo/..").node_is_absolute_posix());
    assert!(!Utf8Path::new("bar/").node_is_absolute_posix());
    assert!(!Utf8Path::new("./baz").node_is_absolute_posix());
  }
}
