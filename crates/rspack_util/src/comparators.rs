use std::cmp::Ordering;

pub fn compare_ids(a: &str, b: &str) -> Ordering {
  let a = a.to_lowercase();
  let b = b.to_lowercase();
  if a < b {
    return Ordering::Less;
  } else if a > b {
    return Ordering::Greater;
  } else {
    return Ordering::Equal;
  }
}

pub fn compare_numbers(a: usize, b: usize) -> Ordering {
  if a < b {
    return Ordering::Less;
  } else if a > b {
    return Ordering::Greater;
  } else {
    return Ordering::Equal;
  }
}
