use std::collections::BTreeMap;

use either::Either;
use rspack_intern::Atom;
use smallvec::SmallVec;

use super::ExportInfoData;

/// Named exports of one module.
///
/// Small modules keep their exports in an array sorted by name, which avoids
/// the BTreeMap node allocation (11 * size_of::<ExportInfoData>(), about 1.4 KiB
/// before the struct was shrunk) that used to dominate the exports info
/// artifact. Inserting into that array memmoves the trailing entries, which is
/// free at a handful of exports but quadratic for a module with thousands of
/// them (a CSS module exposes one class per export, inserted in source order),
/// so once a module outgrows [`INLINE_EXPORTS`] the entries move back into a
/// BTreeMap. Both variants iterate in key order.
#[derive(Debug, Clone)]
pub enum NamedExports {
  Inline(SmallVec<[(Atom, ExportInfoData); 1]>),
  Spilled(BTreeMap<Atom, ExportInfoData>),
}

/// Number of exports kept in the sorted inline vector before spilling to a
/// BTreeMap.
const INLINE_EXPORTS: usize = 16;

impl Default for NamedExports {
  fn default() -> Self {
    Self::Inline(SmallVec::new())
  }
}

impl NamedExports {
  #[inline]
  pub fn len(&self) -> usize {
    match self {
      Self::Inline(exports) => exports.len(),
      Self::Spilled(exports) => exports.len(),
    }
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    match self {
      Self::Inline(exports) => exports.is_empty(),
      Self::Spilled(exports) => exports.is_empty(),
    }
  }

  #[inline]
  pub fn get(&self, name: &Atom) -> Option<&ExportInfoData> {
    match self {
      Self::Inline(exports) => exports
        .binary_search_by(|(key, _)| key.cmp(name))
        .ok()
        .map(|index| &exports[index].1),
      Self::Spilled(exports) => exports.get(name),
    }
  }

  #[inline]
  pub fn get_mut(&mut self, name: &Atom) -> Option<&mut ExportInfoData> {
    match self {
      Self::Inline(exports) => match exports.binary_search_by(|(key, _)| key.cmp(name)) {
        Ok(index) => Some(&mut exports[index].1),
        Err(_) => None,
      },
      Self::Spilled(exports) => exports.get_mut(name),
    }
  }

  pub fn insert(&mut self, name: Atom, info: ExportInfoData) -> Option<ExportInfoData> {
    if let Self::Inline(exports) = self
      && exports.len() >= INLINE_EXPORTS
      && exports.binary_search_by(|(key, _)| key.cmp(&name)).is_err()
    {
      let spilled: BTreeMap<Atom, ExportInfoData> = exports.drain(..).collect();
      *self = Self::Spilled(spilled);
    }
    match self {
      Self::Inline(exports) => match exports.binary_search_by(|(key, _)| key.cmp(&name)) {
        Ok(index) => Some(std::mem::replace(&mut exports[index].1, info)),
        Err(index) => {
          exports.insert(index, (name, info));
          None
        }
      },
      Self::Spilled(exports) => exports.insert(name, info),
    }
  }

  #[inline]
  pub fn remove(&mut self, name: &Atom) -> Option<ExportInfoData> {
    match self {
      Self::Inline(exports) => match exports.binary_search_by(|(key, _)| key.cmp(name)) {
        Ok(index) => Some(exports.remove(index).1),
        Err(_) => None,
      },
      Self::Spilled(exports) => exports.remove(name),
    }
  }

  #[inline]
  pub fn clear(&mut self) {
    match self {
      Self::Inline(exports) => exports.clear(),
      Self::Spilled(exports) => exports.clear(),
    }
  }

  pub fn iter(&self) -> impl Iterator<Item = (&Atom, &ExportInfoData)> {
    match self {
      Self::Inline(exports) => Either::Left(exports.iter().map(|(key, info)| (key, info))),
      Self::Spilled(exports) => Either::Right(exports.iter()),
    }
  }

  pub fn iter_mut(&mut self) -> impl Iterator<Item = (&Atom, &mut ExportInfoData)> {
    match self {
      Self::Inline(exports) => Either::Left(exports.iter_mut().map(|(key, info)| (&*key, info))),
      Self::Spilled(exports) => Either::Right(exports.iter_mut()),
    }
  }

  pub fn values(&self) -> impl Iterator<Item = &ExportInfoData> {
    self.iter().map(|(_, info)| info)
  }

  pub fn values_mut(&mut self) -> impl Iterator<Item = &mut ExportInfoData> {
    self.iter_mut().map(|(_, info)| info)
  }

  pub fn keys(&self) -> impl Iterator<Item = &Atom> {
    self.iter().map(|(key, _)| key)
  }
}
