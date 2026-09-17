use std::{borrow::Borrow, cell::Cell, collections::HashSet, hash::Hash, rc::Rc};

use rspack_collections::SsoHashSet;

#[test]
fn matches_hash_set_across_storage_transitions() {
  let mut actual = SsoHashSet::new();
  let mut expected = HashSet::new();
  for round in 0..8 {
    for value in (0..16).chain(0..16).filter(|value| value % 3 != round % 3) {
      assert_eq!(actual.insert(value), expected.insert(value));
      assert_eq!(actual.len(), expected.len());
    }
    for value in 0..20 {
      assert_eq!(actual.contains(&value), expected.contains(&value));
      if value % 2 == round % 2 {
        assert_eq!(actual.remove(&value), expected.remove(&value));
      }
    }
    assert_eq!(actual.iter().copied().collect::<HashSet<_>>(), expected);
    assert_eq!((&actual).into_iter().count(), expected.len());
    let snapshot = actual.clone();
    actual.clear();
    expected.clear();
    assert!(actual.is_empty());
    assert!(actual.is_subset(&snapshot));
    assert!(!snapshot.is_subset(&actual));
    actual.extend(snapshot);
    expected.extend(actual.iter().copied());
  }
  assert_eq!(actual.into_iter().collect::<HashSet<_>>(), expected);
}

#[test]
fn borrowed_lookup_and_independent_clones() {
  for size in [0, 1, 4, 5, 9] {
    let values: Vec<_> = (0..size).map(|value| value.to_string()).collect();
    let mut set: SsoHashSet<String> = values
      .iter()
      .cloned()
      .chain(values.iter().cloned())
      .collect();
    assert_eq!(set.len(), size);
    let snapshot = set.clone();
    assert!(set.is_subset(&snapshot));
    assert!(snapshot.is_subset(&set));
    for value in &values {
      assert!(set.contains(value.as_str()));
      assert!(set.remove(value.as_str()));
      assert!(!set.remove(value.as_str()));
      assert!(snapshot.contains(value.as_str()));
    }
    assert!(set.is_empty());
    assert_eq!(
      snapshot.into_iter().collect::<HashSet<_>>(),
      values.into_iter().collect::<HashSet<_>>()
    );
  }
}

// Deliberately neither Copy nor Clone: promotion must move owned values.
#[derive(Debug)]
struct OwnedKey {
  value: usize,
  drops: Rc<Cell<usize>>,
}

impl Borrow<usize> for OwnedKey {
  fn borrow(&self) -> &usize {
    &self.value
  }
}

impl PartialEq for OwnedKey {
  fn eq(&self, other: &Self) -> bool {
    self.value == other.value
  }
}

impl Eq for OwnedKey {}

impl Hash for OwnedKey {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.value.hash(state);
  }
}

impl Drop for OwnedKey {
  fn drop(&mut self) {
    self.drops.set(self.drops.get() + 1);
  }
}

#[test]
fn non_clone_values_are_dropped_once() {
  for size in [1, 4, 5, 9] {
    let drops = Rc::new(Cell::new(0));
    let mut set = SsoHashSet::new();
    for value in 0..size {
      assert!(set.insert(OwnedKey {
        value,
        drops: Rc::clone(&drops)
      }));
    }
    assert_eq!(drops.get(), 0);
    assert!(!set.insert(OwnedKey {
      value: 0,
      drops: Rc::clone(&drops)
    }));
    assert_eq!(drops.get(), 1);
    assert!(set.contains(&0));
    assert!(set.remove(&0));
    assert_eq!(drops.get(), 2);
    set.clear();
    assert_eq!(drops.get(), size + 1);
    set.insert(OwnedKey {
      value: 42,
      drops: Rc::clone(&drops),
    });
    drop(set.into_iter());
    assert_eq!(drops.get(), size + 2);
  }
}
