use std::cmp::Ordering;

use rspack_ids::id_helpers::precompute_deterministic_id_candidates;
use rspack_util::number_hash::get_number_hash;

#[test]
fn precompute_deterministic_id_candidates_keeps_sorted_order_and_initial_ids() {
  let items = vec!["module-c", "module-a", "module-b"];

  let prepared = precompute_deterministic_id_candidates(
    items,
    |item: &&str| item.to_string(),
    |a: &&str, b: &&str| if a == b { Ordering::Equal } else { a.cmp(b) },
    1000,
    0,
  );

  let actual = prepared
    .into_iter()
    .map(|prepared| (prepared.item, prepared.name, prepared.initial_id))
    .collect::<Vec<_>>();

  let expected = ["module-a", "module-b", "module-c"]
    .into_iter()
    .map(|item| {
      let name = item.to_string();
      let initial_id = get_number_hash(&format!("{name}0"), 1000);
      (item, name, initial_id)
    })
    .collect::<Vec<_>>();

  assert_eq!(actual, expected);
}
