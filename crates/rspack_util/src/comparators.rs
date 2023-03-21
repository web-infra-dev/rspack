use std::cmp::Ordering;

#[allow(clippy::comparison_chain)]
pub fn compare_ids(a: &str, b: &str) -> Ordering {
  let a = a.to_lowercase();
  let b = b.to_lowercase();
  if a < b {
    Ordering::Less
  } else if a > b {
    Ordering::Greater
  } else {
    Ordering::Equal
  }
}
#[allow(clippy::comparison_chain)]
pub fn compare_numbers(a: usize, b: usize) -> Ordering {
  if a < b {
    Ordering::Less
  } else if a > b {
    Ordering::Greater
  } else {
    Ordering::Equal
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_compare_ids() {
    assert_eq!(compare_ids("abc", "def"), Ordering::Less);
    assert_eq!(compare_ids("DEF", "abc"), Ordering::Greater);
    assert_eq!(compare_ids("abc", "ABC"), Ordering::Equal);
  }

  #[test]
  fn test_compare_numbers() {
    assert_eq!(compare_numbers(1, 2), Ordering::Less);
    assert_eq!(compare_numbers(2, 1), Ordering::Greater);
    assert_eq!(compare_numbers(1, 1), Ordering::Equal);
  }
}
