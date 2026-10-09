use cow_utils::CowUtils;

/// Return whether a character has special meaning in glob patterns.
pub fn is_glob_metacharacter(c: char) -> bool {
  matches!(c, '*' | '?' | '[' | '{')
}

/// Return the byte index of the first unescaped glob metacharacter in a pattern.
fn first_glob_metacharacter_index(pattern: &str) -> Option<usize> {
  let mut escaped = false;
  for (byte_idx, c) in pattern.char_indices() {
    if escaped {
      escaped = false;
      continue;
    }

    if c == '\\' {
      escaped = true;
      continue;
    }

    if is_glob_metacharacter(c) {
      return Some(byte_idx);
    }
  }

  None
}

/// Return the byte index after the base directory prefix of a glob pattern.
pub fn glob_base_dir_end(pattern: &str) -> usize {
  let idx = first_glob_metacharacter_index(pattern).unwrap_or(pattern.len());

  pattern[..idx]
    .rfind('/')
    .map_or(0, |slash_idx| slash_idx + 1)
}

/// Extract the base directory from a glob pattern.
/// Returns everything before the first glob metacharacter, up to and including the last `/`.
pub fn extract_glob_base_dir(pattern: &str) -> &str {
  match glob_base_dir_end(pattern) {
    0 => "./",
    end => &pattern[..end],
  }
}

/// Normalize backslashes to forward slashes in a path string.
pub fn normalize_path_separators(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\\' {
      if chars
        .peek()
        .is_some_and(|next| matches!(next, '*' | '?' | '[' | ']' | '{' | '}'))
      {
        result.push(c);
      } else {
        result.push('/');
      }
    } else {
      result.push(c);
    }
  }
  result
}

/// Normalize backslashes to forward slashes in a literal filesystem path.
pub fn normalize_path_separators_for_path(s: &str) -> String {
  s.cow_replace('\\', "/").into_owned()
}

pub fn unescape_glob_path(s: &str) -> String {
  let mut result = String::with_capacity(s.len());
  let mut chars = s.chars().peekable();
  while let Some(c) = chars.next() {
    if c == '\\'
      && chars
        .peek()
        .is_some_and(|next| matches!(next, '*' | '?' | '[' | ']' | '{' | '}'))
    {
      if let Some(next) = chars.next() {
        result.push(next);
      }
    } else {
      result.push(c);
    }
  }
  result
}
