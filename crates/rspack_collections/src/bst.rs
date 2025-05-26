use std::borrow::Borrow;
use std::iter::{FromIterator, IntoIterator, Map};
use std::mem;
use rspack_cacheable::{ with, DeserializeError };

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BstMap<K, V> {
    list: Vec<(K, V)>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BstSet<T>(BstMap<T, ()>);

pub enum Entry<'map, K, V> {
    Vacant(VacantEntry<'map, K, V>),
    Occupied(OccupiedEntry<'map, K, V>),
}

pub struct VacantEntry<'map, K, V> {
    map: &'map mut BstMap<K, V>,
    index: usize,
    key: K,
}

pub struct OccupiedEntry<'map, K, V> {
    map: &'map mut BstMap<K, V>,
    index: usize,
}

impl<K, V> BstMap<K, V> {
    pub fn with_capacity(cap: usize) -> Self {
        BstMap { list: Vec::with_capacity(cap) }
    }
    
    pub fn insert(&mut self, k: K, v: V) -> Option<V>
    where
        K: Ord,
    {
        match self.list.binary_search_by(|(k2, _)| k2.cmp(&k)) {
            Ok(idx) => {
                let slot = self.list.get_mut(idx)?;
                Some(mem::replace(slot, (k, v)).1)
            }
            Err(idx) => {
                self.list.insert(idx, (k, v));
                None
            }
        }
    }

    pub fn get<Q>(&self, q: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.list
            .binary_search_by(|(k, _)| k.borrow().cmp(&q))
            .ok()
            .and_then(|idx| self.list.get(idx))
            .map(|(_, v)| v)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.list.iter().map(|(_, v)| v)
    }

    pub fn entry(&mut self, k: K) -> Entry<'_, K, V>
    where
        K: Ord,
    {
        match self.list.binary_search_by(|(k2, _)| k2.cmp(&k)) {
            Ok(idx) => Entry::Occupied(OccupiedEntry { map: self, index: idx }),
            Err(idx) => Entry::Vacant(VacantEntry { map: self, index: idx, key: k }),
        }
    }
}

impl<'map, K, V> VacantEntry<'map, K, V> {
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn insert(self, value: V) -> &'map mut V {
        self.map.list.insert(self.index, (self.key, value));
        &mut self.map.list[self.index].1
    }
}

impl<K, V> OccupiedEntry<'_, K, V> {
    pub fn key(&self) -> &K {
        &self.map.list[self.index].0
    }

    pub fn get(&self) -> &V {
        &self.map.list[self.index].1
    }

    pub fn get_mut(&mut self) -> &mut V {
        &mut self.map.list[self.index].1
    }

    pub fn insert(&mut self, value: V) -> V {
        mem::replace(&mut self.map.list[self.index].1, value)
    }
}

impl<K, V> Default for BstMap<K, V> {
    fn default() -> Self {
        BstMap { list: Vec::new() }
    }
}

impl<'a, K, V> IntoIterator for &'a BstMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Map<std::slice::Iter<'a, (K, V)>, fn(&'a (K, V)) -> (&'a K, &'a V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.list.iter().map(|(k, v)| (k, v))
    }
}

impl<K, V> IntoIterator for BstMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;

    fn into_iter(self) -> Self::IntoIter {
        self.list.into_iter()
    }
}

impl<K, V> Extend<(K, V)> for BstMap<K, V>
where
    K: Ord,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (K, V)>,
    {
        self.list.extend(iter);
        self.list.sort_by(|(k, _), (k2, _)| k.cmp(k2));
        self.list.dedup_by(|(k, _), (k2, _)| k == k2);
    }
}

impl<K, V> FromIterator<(K, V)> for BstMap<K, V>
where
    K: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut map = BstMap::default();
        map.extend(iter);
        map
    }
}

impl<T> BstSet<T> {
    pub fn with_capacity(cap: usize) -> Self {
        BstSet(BstMap::with_capacity(cap))
    }

    /// # NOTE
    ///
    /// If the iterator is not in order, there will be a logic error.
    /// but this does not cause memory safety issues.
    pub fn from_sorted_and_dedup_iter<I>(iter: I)
        -> Self
    where
        I: Iterator<Item = T>,
        T: Ord
    {
        BstSet(BstMap { list: iter.map(|k| (k, ())).collect() })
    }    
    
    pub fn len(&self) -> usize {
        self.0.list.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, value: T) -> bool
    where
        T: Ord,
    {
        match self.0.entry(value) {
            Entry::Vacant(entry) => {
                entry.insert(());
                true
            }
            Entry::Occupied(_) => false,
        }
    }

    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q> + Ord,
        Q: Ord + ?Sized
    {
        self.0.get(value).is_some()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> {
        self.into_iter()
    }

    pub fn difference<'a>(&'a self, other: &'a Self)
        -> impl Iterator<Item = &'a T>
    where
        T: Ord
    {
        self.iter().filter(|k| !other.contains(k))
    }

    pub fn intersection<'a>(&'a self, other: &'a Self)
        -> impl Iterator<Item = &'a T>
    where
        T: Ord
    {
        let (base, diff) = if self.len() <= other.len() {
            (self, other)
        } else {
            (other, self)
        };

        base.iter().filter(|k| diff.contains(k))
    }
}

impl<T> Default for BstSet<T> {
    fn default() -> Self {
        BstSet(Default::default())
    }
}

impl<'a, T> IntoIterator for &'a BstSet<T> {
    type Item = &'a T;
    type IntoIter = Map<std::slice::Iter<'a, (T, ())>, fn(&'a (T, ())) -> &'a T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.list.iter().map(|(k, _)| k)
    }
}

impl<T> IntoIterator for BstSet<T> {
    type Item = T;
    type IntoIter = Map<std::vec::IntoIter<(T, ())>, fn((T, ())) -> T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.list.into_iter().map(|(k, _)| k)
    }
}

impl<T> Extend<T> for BstSet<T>
where
    T: Ord,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.0.extend(iter.into_iter().map(|k| (k, ())));
    }
}

impl<T> FromIterator<T> for BstSet<T>
where
    T: Ord,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut set = BstSet::default();
        set.extend(iter);
        set
    }
}

impl<T> with::AsVecConverter for BstSet<T>
where
  T: Ord,
{
  type Item = T;
  fn len(&self) -> usize {
    self.len()
  }
  fn iter(&self) -> impl Iterator<Item = &Self::Item> {
    self.iter()
  }
  fn from(
    data: impl Iterator<Item = Result<Self::Item, DeserializeError>>,
  ) -> Result<Self, DeserializeError> {
    data.collect()
  }
}
