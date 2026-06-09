use std::{borrow::Cow, fmt::Debug};

use cow_utils::CowUtils;
use fast_glob::glob_match;
use regex::Regex;
use rspack_regex::RspackRegex;

#[derive(Default)]
pub enum FsWatcherIgnored {
  #[default]
  None,
  Path(String),
  Paths(Vec<String>),
  Regex(RspackRegex),
}

impl Debug for FsWatcherIgnored {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FsWatcherIgnored::None => write!(f, "FsWatcherIgnored::None"),
      FsWatcherIgnored::Path(s) => write!(f, "FsWatcherIgnored::Path({s})"),
      FsWatcherIgnored::Paths(s) => write!(f, "FsWatcherIgnored::Paths({s:?})"),
      FsWatcherIgnored::Regex(reg) => write!(f, "FsWatcherIgnored::Regex({reg:?})"),
    }
  }
}

/// Normalize the path by replacing backslashes with forward slashes.
/// Smooth out the differences in the system, specifically for Windows
fn normalize_path<'a>(path: &'a str) -> Cow<'a, str> {
  path.cow_replace("\\", "/")
}

impl FsWatcherIgnored {
  pub fn should_be_ignored(&self, p: &str) -> bool {
    match self {
      FsWatcherIgnored::None => false,
      FsWatcherIgnored::Path(path) => glob_match(path, normalize_path(p).as_bytes()),
      FsWatcherIgnored::Paths(paths) => paths
        .iter()
        .any(|path| glob_match(path, normalize_path(p).as_bytes())),

      FsWatcherIgnored::Regex(reg) => reg.test(&normalize_path(p)),
    }
  }
}

/// Translate a glob to a regex matching the named path AND anything nested
/// under it — watchpack's `(?:$|/)` trick, so a directory match covers its
/// whole subtree without an ancestor walk.
fn glob_to_subtree_regex(glob: &str) -> String {
  let glob = glob.cow_replace('\\', "/");
  let bytes = glob.as_bytes();
  let mut src = String::from("^");
  let mut i = 0;
  while i < bytes.len() {
    match bytes[i] {
      b'*' if bytes.get(i + 1) == Some(&b'*') => {
        i += 1;
        if bytes.get(i + 1) == Some(&b'/') {
          i += 1;
          src.push_str("(?:.*/)?"); // `**/` → any leading directories
        } else {
          src.push_str(".*"); // trailing `**`
        }
      }
      b'*' => src.push_str("[^/]*"),
      b'?' => src.push_str("[^/]"),
      c @ (b'.' | b'+' | b'(' | b')' | b'[' | b']' | b'{' | b'}' | b'^' | b'$' | b'|') => {
        src.push('\\');
        src.push(c as char);
      }
      c => src.push(c as char),
    }
    i += 1;
  }
  src.push_str("(?:$|/)");
  src
}

/// watchpack-style ignore matcher: all globs are rewritten to match their
/// subtree and folded into one precompiled regex, so a single `is_match`
/// classifies an event path. The user-supplied `Regex` variant is applied as-is.
#[derive(Default)]
pub struct IgnoredMatcher {
  globs: Option<Regex>,
  user_regex: Option<RspackRegex>,
}

impl IgnoredMatcher {
  pub fn new(ignored: FsWatcherIgnored) -> Self {
    fn compile(patterns: &[String]) -> Option<Regex> {
      let parts: Vec<String> = patterns
        .iter()
        .filter(|g| !g.is_empty())
        .map(|g| format!("(?:{})", glob_to_subtree_regex(g)))
        .collect();
      if parts.is_empty() {
        return None;
      }
      match Regex::new(&parts.join("|")) {
        Ok(re) => Some(re),
        // Glob escaping guarantees valid syntax, so the only realistic failure
        // is the regex size limit on a pathological `ignored` config. Degrade
        // to "no glob filtering" (events flow, no missed changes) but surface
        // it — never disable ignores silently.
        Err(e) => {
          tracing::error!("failed to compile ignored patterns, ignore filtering disabled: {e}");
          None
        }
      }
    }
    match ignored {
      FsWatcherIgnored::None => Self::default(),
      FsWatcherIgnored::Path(p) => Self {
        globs: compile(&[p]),
        user_regex: None,
      },
      FsWatcherIgnored::Paths(ps) => Self {
        globs: compile(&ps),
        user_regex: None,
      },
      FsWatcherIgnored::Regex(reg) => Self {
        globs: None,
        user_regex: Some(reg),
      },
    }
  }

  /// Whether `path` is ignored — directly or by living inside an ignored
  /// directory. Single regex test (plus the optional user regex).
  pub fn is_ignored(&self, path: &str) -> bool {
    if self.globs.is_none() && self.user_regex.is_none() {
      return false; // no patterns — skip per-event normalization
    }
    let path = normalize_path(path);
    self.globs.as_ref().is_some_and(|re| re.is_match(&path))
      || self.user_regex.as_ref().is_some_and(|re| re.test(&path))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn matcher(pattern: &str) -> IgnoredMatcher {
    IgnoredMatcher::new(FsWatcherIgnored::Path(pattern.to_owned()))
  }

  #[test]
  fn subtree_match_catches_directory_and_its_files() {
    let m = matcher("**/dist/.rstest-temp");
    assert!(m.is_ignored("/proj/dist/.rstest-temp"));
    assert!(m.is_ignored("/proj/dist/.rstest-temp/foo.mjs"));
    assert!(m.is_ignored("/proj/dist/.rstest-temp/nested/bar.mjs"));
  }

  #[test]
  fn subtree_match_keeps_unrelated_paths() {
    let m = matcher("**/dist/.rstest-temp");
    assert!(!m.is_ignored("/proj/src/index.js"));
    assert!(!m.is_ignored("/proj/dist/main.js"));
    // Must not match a sibling directory that merely shares a prefix.
    assert!(!m.is_ignored("/proj/dist/.rstest-temp-old/x.js"));
  }

  #[test]
  fn none_matches_nothing() {
    assert!(!IgnoredMatcher::default().is_ignored("/anything/at/all"));
  }

  #[test]
  fn windows_form_paths_match_after_normalization() {
    // Windows delivers backslash, drive-letter paths; `is_ignored` normalizes
    // the haystack, so matching is separator-agnostic. Plain-string input keeps
    // this portable — it runs on any host.
    let nm = matcher("**/node_modules");
    assert!(nm.is_ignored(r"C:\proj\node_modules\pkg\index.js"));
    assert!(nm.is_ignored(r"C:\proj\packages\app\node_modules\dep\lib.js"));
    assert!(!nm.is_ignored(r"C:\proj\src\index.ts"));

    let temp = matcher("**/dist/.rstest-temp");
    assert!(temp.is_ignored(r"C:\proj\dist\.rstest-temp\spec.test.mjs"));
    assert!(!temp.is_ignored(r"C:\proj\dist\main.js"));
    // mixed separators must work too
    assert!(temp.is_ignored("C:/proj/dist/.rstest-temp/x.mjs"));
  }
}
