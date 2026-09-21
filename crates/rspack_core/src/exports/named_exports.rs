use rspack_intern::Atom;
use smallvec::SmallVec;

use super::ExportInfoData;

/// Named exports of one module, kept as an array sorted by name.
///
/// A BTreeMap node allocates 11 * size_of::<ExportInfoData>() bytes as soon as
/// the first export is inserted, which is around 1.4 KiB while most modules
/// only have a handful of exports, so the map node dominates the footprint of
/// the exports info artifact. A sorted SmallVec keeps the common case
/// allocation-free and preserves the key-ordered iteration the previous
/// container provided.
#[derive(Debug, Clone, Default)]
pub struct NamedExports(SmallVec<[(Atom, ExportInfoData); 1]>);

impl NamedExports {
  #[inline]
  pub fn len(&self) -> usize {
    self.0.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  #[inline]
  fn index_of(&self, name: &Atom) -> Option<usize> {
    self.0.binary_search_by(|(key, _)| key.cmp(name)).ok()
  }

  #[inline]
  pub fn get(&self, name: &Atom) -> Option<&ExportInfoData> {
    self.index_of(name).map(|index| &self.0[index].1)
  }

  #[inline]
  pub fn get_mut(&mut self, name: &Atom) -> Option<&mut ExportInfoData> {
    match self.index_of(name) {
      Some(index) => Some(&mut self.0[index].1),
      None => None,
    }
  }

  #[inline]
  pub fn insert(&mut self, name: Atom, info: ExportInfoData) -> Option<ExportInfoData> {
    match self.0.binary_search_by(|(key, _)| key.cmp(&name)) {
      Ok(index) => Some(std::mem::replace(&mut self.0[index].1, info)),
      Err(index) => {
        self.0.insert(index, (name, info));
        None
      }
    }
  }

  #[inline]
  pub fn remove(&mut self, name: &Atom) -> Option<ExportInfoData> {
    match self.index_of(name) {
      Some(index) => Some(self.0.remove(index).1),
      None => None,
    }
  }

  #[inline]
  pub fn clear(&mut self) {
    self.0.clear();
  }

  pub fn iter(&self) -> impl Iterator<Item = (&Atom, &ExportInfoData)> {
    self.0.iter().map(|(key, info)| (key, info))
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Atom, &mut ExportInfoData)> {
    self.0.iter_mut().map(|(key, info)| (&*key, info))
  }

  pub fn values(&self) -> impl Iterator<Item = &ExportInfoData> {
    self.0.iter().map(|(_, info)| info)
  }

  pub fn values_mut(&mut self) -> impl Iterator<Item = &mut ExportInfoData> {
    self.0.iter_mut().map(|(_, info)| info)
  }

  pub fn keys(&self) -> impl Iterator<Item = &Atom> {
    self.0.iter().map(|(key, _)| key)
  }
}
