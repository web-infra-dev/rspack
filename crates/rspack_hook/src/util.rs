pub fn sort_push<T, K: Ord>(vec: &mut Vec<T>, e: T, mut f: impl FnMut(&T) -> K) {
  if let Some(last) = vec.last() {
    let e_key = f(&e);
    let cmp = e_key.cmp(&f(last));
    if cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal {
      vec.push(e);
    } else {
      let insert_at = match vec.binary_search_by_key(&e_key, f) {
        Ok(insert_at) | Err(insert_at) => insert_at,
      };
      vec.insert(insert_at, e);
    }
  } else {
    vec.push(e);
  }
}
