use std::borrow::Cow;

use cow_utils::CowUtils;

/// Replace every backslash with `/`, including before glob metacharacters.
/// Accepts Windows-form paths on every platform and borrows unchanged input.
pub fn normalize_path_separators(path: &str) -> Cow<'_, str> {
  path.cow_replace('\\', "/")
}

/// Normalize a native filesystem path for forward-slash matching.
/// On Unix, backslashes are literal filename characters and remain unchanged.
pub fn normalize_native_path_separators(path: &str) -> Cow<'_, str> {
  if cfg!(windows) {
    normalize_path_separators(path)
  } else {
    Cow::Borrowed(path)
  }
}
