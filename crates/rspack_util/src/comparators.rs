use std::cmp::Ordering;

pub fn compare_ids(a: &str, b: &str) -> Ordering {
  unicase::UniCase::new(a).cmp(&unicase::UniCase::new(b))
}

pub fn compare_numbers(a: u32, b: u32) -> Ordering {
  a.cmp(&b)
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
