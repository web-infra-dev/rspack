use rspack_util::atom::Atom;
use rustc_hash::FxHashMap;

use super::ExportInfoData;

/// Map from export name to [`ExportInfoData`] with O(1) name lookup and
/// name-sorted iteration.
///
/// Name lookup is on the hot path of export analysis and scales with the
/// number of dependencies, while sorted iteration is needed for deterministic
/// hashing and code generation. A `BTreeMap` makes every lookup pay O(log n)
/// string comparisons, so instead entries are stored in an append-only vector
/// with a hash index by name plus a name-sorted index list.
#[derive(Debug, Clone, Default)]
pub struct ExportsInfoMap {
  /// Export entries in insertion order. Entries are never removed.
  entries: Vec<ExportInfoData>,
  /// Export name of each entry in `entries`.
  names: Vec<Atom>,
  /// name -> index into `entries`.
  index: FxHashMap<Atom, u32>,
  /// Indices of `entries` sorted by export name.
  sorted: Vec<u32>,
}

impl ExportsInfoMap {
  pub fn len(&self) -> usize {
    self.entries.len()
  }

  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  pub fn contains_key(&self, name: &Atom) -> bool {
    self.index.contains_key(name)
  }

  pub fn get(&self, name: &Atom) -> Option<&ExportInfoData> {
    let i = *self.index.get(name)?;
    Some(&self.entries[i as usize])
  }

  pub fn get_mut(&mut self, name: &Atom) -> Option<&mut ExportInfoData> {
    let i = *self.index.get(name)?;
    Some(&mut self.entries[i as usize])
  }

  pub fn insert(&mut self, name: Atom, value: ExportInfoData) -> Option<ExportInfoData> {
    if let Some(&i) = self.index.get(&name) {
      return Some(std::mem::replace(&mut self.entries[i as usize], value));
    }
    let i = u32::try_from(self.entries.len()).expect("too many exports");
    let pos = self
      .sorted
      .binary_search_by(|&j| self.names[j as usize].cmp(&name))
      .expect_err("name is not in the map");
    self.entries.push(value);
    self.names.push(name.clone());
    self.index.insert(name, i);
    self.sorted.insert(pos, i);
    None
  }

  /// Values in export-name order.
  pub fn values(&self) -> impl DoubleEndedIterator<Item = &ExportInfoData> + ExactSizeIterator {
    self.sorted.iter().map(|&i| &self.entries[i as usize])
  }

  /// Values in unspecified order; use only for order-independent mutations.
  pub fn values_mut(
    &mut self,
  ) -> impl DoubleEndedIterator<Item = &mut ExportInfoData> + ExactSizeIterator {
    self.entries.iter_mut()
  }

  /// Entries in export-name order.
  pub fn iter(
    &self,
  ) -> impl DoubleEndedIterator<Item = (&Atom, &ExportInfoData)> + ExactSizeIterator {
    self
      .sorted
      .iter()
      .map(|&i| (&self.names[i as usize], &self.entries[i as usize]))
  }

  /// Export names in export-name order.
  pub fn keys(&self) -> impl DoubleEndedIterator<Item = &Atom> + ExactSizeIterator {
    self.sorted.iter().map(|&i| &self.names[i as usize])
  }
}

#[cfg(test)]
mod tests {
  use super::{super::ExportsInfo, ExportsInfoMap};
  use crate::ExportInfoData;

  fn new_info(name: &str) -> ExportInfoData {
    ExportInfoData::new(ExportsInfo::new(), Some(name.into()), None)
  }

  #[test]
  fn sorted_iteration_and_lookup() {
    let mut map = ExportsInfoMap::default();
    assert!(map.is_empty());

    map.insert("b".into(), new_info("b"));
    map.insert("a".into(), new_info("a"));
    map.insert("c".into(), new_info("c"));

    assert_eq!(map.len(), 3);
    assert!(map.contains_key(&"a".into()));
    assert!(map.get(&"missing".into()).is_none());

    let names: Vec<_> = map.keys().map(|name| name.as_str()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
    let values: Vec<_> = map
      .values()
      .map(|info| info.name().expect("should have name").as_str())
      .collect();
    assert_eq!(values, vec!["a", "b", "c"]);
  }

  #[test]
  fn insert_replaces_existing() {
    let mut map = ExportsInfoMap::default();
    assert!(map.insert("a".into(), new_info("a")).is_none());
    assert!(map.insert("a".into(), new_info("a")).is_some());
    assert_eq!(map.len(), 1);
  }
}
