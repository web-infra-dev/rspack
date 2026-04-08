use std::cmp::Ordering;

use rspack_ids::id_helpers::{
  assign_deterministic_ids_from_precomputed_candidates, precompute_deterministic_id_candidates,
};
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

#[test]
fn assign_deterministic_ids_from_precomputed_candidates_retries_conflicts_in_order() {
  let range = 1000;
  let salt = 10;
  let prepared = precompute_deterministic_id_candidates(
    vec!["chunk-c", "chunk-a", "chunk-b"],
    |item: &&str| item.to_string(),
    |a: &&str, b: &&str| if a == b { Ordering::Equal } else { a.cmp(b) },
    range,
    salt,
  );

  let first_initial_id = prepared[0].initial_id;
  let first_retry_id = get_number_hash(&format!("{}{}", prepared[0].name, salt + 1), range);
  let second_initial_id = prepared[1].initial_id;
  let third_initial_id = prepared[2].initial_id;

  assert_ne!(first_initial_id, first_retry_id);
  assert_ne!(first_retry_id, second_initial_id);
  assert_ne!(first_retry_id, third_initial_id);
  assert_ne!(second_initial_id, third_initial_id);

  let mut assigned = Vec::new();
  let mut used_ids = [first_initial_id.to_string()].into_iter().collect();

  assign_deterministic_ids_from_precomputed_candidates(
    prepared,
    &mut used_ids,
    range,
    salt,
    |item, id| assigned.push((item, id)),
  );

  assert_eq!(
    assigned,
    vec![
      ("chunk-a", first_retry_id),
      ("chunk-b", second_initial_id),
      ("chunk-c", third_initial_id),
    ]
  );
}
