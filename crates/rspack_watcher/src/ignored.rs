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

/// Faithful port of watchpack's `lib/util/globToRegExp.js` (specialized for
/// `extended` + `globstar`, see webpack/watchpack#312): translates a glob to a
/// regex source WITHOUT anchors. `compile` adds the `^` and the `(?:$|/)`
/// subtree suffix, exactly as watchpack's `stringToRegexp` does — so a
/// directory match also covers its whole subtree. Keeping it in lockstep with
/// watchpack means `{a,b}` brace groups, `[...]` classes, `?` and globstar all
/// behave identically to the watcher rspack mirrors.
fn glob_to_regexp(glob: &str) -> String {
  let bytes = glob.as_bytes();
  let len = bytes.len();
  let mut re = String::new();
  let mut in_group = false;
  // Start of the current run of literal characters, copied with one slice.
  let mut literal_start = 0;
  let mut i = 0;
  while i < len {
    let token_start = i;
    let mapped: &str = match bytes[i] {
      b'/' => "\\/",
      b'$' => "\\$",
      b'^' => "\\^",
      b'+' => "\\+",
      b'.' => "\\.",
      b'(' => "\\(",
      b')' => "\\)",
      b'=' => "\\=",
      b'!' => "\\!",
      b'|' => "\\|",
      b'?' => ".",
      b'[' => "[",
      b']' => "]",
      b'{' => {
        in_group = true;
        "("
      }
      b'}' => {
        in_group = false;
        ")"
      }
      b',' => {
        if in_group {
          "|"
        } else {
          "\\,"
        }
      }
      b'*' => {
        let at_start = i == 0;
        let after_slash = !at_start && bytes[i - 1] == b'/';
        while i + 1 < len && bytes[i + 1] == b'*' {
          i += 1;
        }
        let multi_star = i > token_start;
        let at_end = i + 1 == len;
        let before_slash = !at_end && bytes[i + 1] == b'/';
        if multi_star && (at_start || after_slash) && (at_end || before_slash) {
          i += 1; // consume the trailing "/" — globstar spans whole segments
          "((?:[^/]*(?:\\/|$))*)"
        } else {
          "([^/]*)"
        }
      }
      // Literal character — extend the current run, flushed lazily.
      _ => {
        i += 1;
        continue;
      }
    };
    if literal_start < token_start {
      re.push_str(&glob[literal_start..token_start]);
    }
    re.push_str(mapped);
    literal_start = i + 1;
    i += 1;
  }
  if literal_start < len {
    re.push_str(&glob[literal_start..]);
  }
  re
}

/// watchpack-style ignore matcher. Exactly one classification strategy is live
/// per watch: glob patterns are rewritten to match their subtree and folded
/// into one precompiled regex (a single `is_match` per event), while a
/// user-supplied `Regex` is applied as-is. `None` short-circuits before
/// normalizing the path.
#[derive(Default)]
pub enum IgnoredMatcher {
  #[default]
  None,
  Globs(Regex),
  Regex(RspackRegex),
}

impl IgnoredMatcher {
  pub fn new(ignored: FsWatcherIgnored) -> Self {
    fn compile(patterns: &[String]) -> IgnoredMatcher {
      let parts: Vec<String> = patterns
        .iter()
        .filter(|g| !g.is_empty())
        .map(|g| {
          // watchpack assumes forward-slash globs; rspack may hand us
          // Windows-form absolute paths, so normalize separators before
          // translating — `is_ignored` normalizes the haystack the same way.
          let g = g.cow_replace('\\', "/");
          format!("(?:^{}(?:$|/))", glob_to_regexp(&g))
        })
        .collect();
      if parts.is_empty() {
        return IgnoredMatcher::None;
      }
      match Regex::new(&parts.join("|")) {
        Ok(re) => IgnoredMatcher::Globs(re),
        // Glob escaping guarantees valid syntax, so the only realistic failure
        // is the regex size limit on a pathological `ignored` config. Degrade
        // to "no glob filtering" (events flow, no missed changes) but surface
        // it — never disable ignores silently.
        Err(e) => {
          tracing::error!("failed to compile ignored patterns, ignore filtering disabled: {e}");
          IgnoredMatcher::None
        }
      }
    }
    match ignored {
      FsWatcherIgnored::None => IgnoredMatcher::None,
      FsWatcherIgnored::Path(p) => compile(&[p]),
      FsWatcherIgnored::Paths(ps) => compile(&ps),
      FsWatcherIgnored::Regex(reg) => IgnoredMatcher::Regex(reg),
    }
  }

  /// Whether `path` is ignored — directly or by living inside an ignored
  /// directory. Single regex test against the normalized path.
  pub fn is_ignored(&self, path: &str) -> bool {
    match self {
      IgnoredMatcher::None => false,
      IgnoredMatcher::Globs(re) => re.is_match(&normalize_path(path)),
      IgnoredMatcher::Regex(re) => re.test(&normalize_path(path)),
    }
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
  fn combines_multiple_globs() {
    // watchpack: `ignored: ["**/foo", "**/bar"]` — folded into one regex.
    let m = IgnoredMatcher::new(FsWatcherIgnored::Paths(vec![
      "**/foo".to_owned(),
      "**/bar".to_owned(),
    ]));
    assert!(m.is_ignored("/x/foo"));
    assert!(m.is_ignored("/x/bar"));
    assert!(!m.is_ignored("/x/baz"));
  }

  #[test]
  fn empty_patterns_match_nothing() {
    // watchpack treats "", [] and an all-empty array as ignoring nothing.
    let cases = [
      FsWatcherIgnored::Path(String::new()),
      FsWatcherIgnored::Paths(vec![]),
      FsWatcherIgnored::Paths(vec![String::new(), String::new()]),
    ];
    for ignored in cases {
      assert!(!IgnoredMatcher::new(ignored).is_ignored("any"));
    }
  }

  #[test]
  fn user_regex_matches_like_watchpack() {
    // watchpack: `ignored: /ignoredPattern/` is applied to the path as-is.
    let m = IgnoredMatcher::new(FsWatcherIgnored::Regex(
      RspackRegex::new("ignoredPattern").unwrap(),
    ));
    assert!(m.is_ignored("/foo/ignoredPattern/bar"));
    assert!(!m.is_ignored("/foo/keep"));
  }

  #[test]
  fn extended_glob_syntax_matches_watchpack() {
    // The full port gives us brace groups, character classes and `?` — the
    // syntax the minimal translator used to swallow as literals.
    let braces = matcher("**/*.{js,ts}");
    assert!(braces.is_ignored("/p/a.js"));
    assert!(braces.is_ignored("/p/pkg/b.ts"));
    assert!(!braces.is_ignored("/p/a.css"));

    let class = matcher("**/foo[0-9]");
    assert!(class.is_ignored("/p/foo1"));
    assert!(!class.is_ignored("/p/fooX"));

    let question = matcher("**/a?c");
    assert!(question.is_ignored("/p/abc"));
    assert!(!question.is_ignored("/p/ac"));
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
