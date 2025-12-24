use rspack_error::{Result, error};
use rustc_hash::FxHashSet as HashSet;

use super::debug_info::DebugInfo;

/// Compare two iterators and return error if they don't match
pub fn ensure_iter_equal<T>(
  compare_name: &str,
  iter1: impl Iterator<Item = T>,
  iter2: impl Iterator<Item = T>,
  debug_info: &DebugInfo,
) -> Result<()>
where
  T: std::fmt::Debug + std::hash::Hash + Eq,
{
  let set1: HashSet<T> = iter1.collect();
  let set2: HashSet<T> = iter2.collect();
  if set1 != set2 {
    // Find items only in set1
    let only_in_1: Vec<_> = set1.difference(&set2).collect();
    // Find items only in set2
    let only_in_2: Vec<_> = set2.difference(&set1).collect();

    let mut error_msg = format!("{} do not match:\n", compare_name);

    if !only_in_1.is_empty() {
      error_msg.push_str(&format!("  Only in path1: {:?}\n", only_in_1));
    }

    if !only_in_2.is_empty() {
      error_msg.push_str(&format!("  Only in path2: {:?}\n", only_in_2));
    }

    error_msg.push_str(&debug_info.to_string());

    return Err(error!(error_msg));
  }

  Ok(())
}
