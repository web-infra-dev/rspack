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
