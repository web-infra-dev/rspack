pub mod undo_log;
use std::{
  borrow::{Borrow, BorrowMut},
  fmt::Debug,
  hash::Hash,
  marker::PhantomData,
  ops,
};

use rayon::iter::IntoParallelRefMutIterator as RayonIntoParallelRefMutIterator;
use rustc_hash::FxHashMap;
pub use undo_log::{Rollback, Snapshot, Snapshots, UndoLogs, VecLog};

pub type SnapshotMapStorage<K, V> = SnapshotMap<K, V, FxHashMap<K, V>, ()>;
pub type SnapshotMapRef<'a, K, V, L> = SnapshotMap<K, V, &'a mut FxHashMap<K, V>, &'a mut L>;
static SNAPSHOT_MAP_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
#[derive(Clone, Debug)]
pub struct SnapshotMap<K, V, M = FxHashMap<K, V>, L = VecLog<UndoLog<K, V>>> {
  map: M,
  undo_log: L,
  _marker: PhantomData<(K, V)>,
  // snapshot stack
  snapshot: Vec<Snapshot>,
  id: u32,
}

// HACK(eddyb) manual impl avoids `Default` bounds on `K` and `V`.
impl<K, V, M, L> Default for SnapshotMap<K, V, M, L>
where
  M: Default,
  L: Default,
{
  fn default() -> Self {
    SnapshotMap {
      map: Default::default(),
      undo_log: Default::default(),
      _marker: PhantomData,
      snapshot: Vec::new(),
      id: SNAPSHOT_MAP_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
    }
  }
}

#[derive(Clone, Debug)]
pub enum UndoLog<K, V>
where
  K: Debug,
  V: Debug,
{
  Inserted(K),
  Overwrite(K, V),
  Purged,
}

impl<K, V, M, L> SnapshotMap<K, V, M, L>
where
  K: Hash + Clone + Eq + Debug,
  V: Debug,
  M: BorrowMut<FxHashMap<K, V>> + Borrow<FxHashMap<K, V>>,
  L: UndoLogs<UndoLog<K, V>>,
{
  pub fn clear(&mut self) {
    self.map.borrow_mut().clear();
    self.undo_log.clear();
  }

  pub fn insert(&mut self, key: K, value: V) -> bool {
    match self.map.borrow_mut().insert(key.clone(), value) {
      None => {
        self.undo_log.push(UndoLog::Inserted(key));
        true
      }
      Some(old_value) => {
        self.undo_log.push(UndoLog::Overwrite(key, old_value));
        false
      }
    }
  }

  pub fn remove(&mut self, key: K) -> bool {
    match self.map.borrow_mut().remove(&key) {
      Some(old_value) => {
        self.undo_log.push(UndoLog::Overwrite(key, old_value));
        true
      }
      None => false,
    }
  }

  pub fn get(&self, key: &K) -> Option<&V> {
    self.map.borrow().get(key)
  }
  pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
    self.map.borrow_mut().get_mut(key)
  }
  pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
    self.map.borrow().iter()
  }
  pub fn for_each<F>(&self, mut f: F)
  where
    F: FnMut(&K, &V),
  {
    for (k, v) in self.map.borrow().iter() {
      f(k, v);
    }
  }
}

impl<K, V> SnapshotMap<K, V>
where
  K: Hash + Clone + Eq + Debug,
  V: Debug,
{
  pub fn save(&mut self) {
    let snapshot = self.snapshot();
    self.snapshot.push(snapshot);
  }
  pub fn recover(&mut self) {
    let snapshot = self.snapshot.pop().unwrap();
    self.rollback_to(snapshot);
  }
  fn snapshot(&mut self) -> Snapshot {
    self.undo_log.start_snapshot()
  }
  fn commit(&mut self, snapshot: Snapshot) {
    self.undo_log.commit(snapshot)
  }

  fn rollback_to(&mut self, snapshot: Snapshot) {
    let map = &mut self.map;
    self.undo_log.rollback_to(|| map, snapshot)
  }
}

impl<'k, K, V, M, L> ops::Index<&'k K> for SnapshotMap<K, V, M, L>
where
  K: Hash + Clone + Eq,
  M: Borrow<FxHashMap<K, V>>,
{
  type Output = V;
  fn index(&self, key: &'k K) -> &V {
    &self.map.borrow()[key]
  }
}

impl<K, V, M, L> Rollback<UndoLog<K, V>> for SnapshotMap<K, V, M, L>
where
  K: Eq + Hash + Debug,
  V: Debug,
  M: Rollback<UndoLog<K, V>>,
{
  fn reverse(&mut self, undo: UndoLog<K, V>) {
    self.map.reverse(undo)
  }
}

impl<K, V> Rollback<UndoLog<K, V>> for FxHashMap<K, V>
where
  K: Eq + Hash + Debug,
  V: Debug,
{
  fn reverse(&mut self, undo: UndoLog<K, V>) {
    match undo {
      UndoLog::Inserted(key) => {
        self.remove(&key);
      }

      UndoLog::Overwrite(key, old_value) => {
        self.insert(key, old_value);
      }

      UndoLog::Purged => {}
    }
  }
}

impl<'data, K, V, M, L> RayonIntoParallelRefMutIterator<'data> for SnapshotMap<K, V, M, L>
where
  K: Eq + Hash + Send + Sync + 'data,
  V: Send + 'data,
  M: BorrowMut<FxHashMap<K, V>> + Send + Sync + 'data,
  L: Send + 'data,
{
  type Item = (&'data K, &'data mut V);
  type Iter = rayon::collections::hash_map::IterMut<'data, K, V>;

  fn par_iter_mut(&'data mut self) -> Self::Iter {
    self.map.borrow_mut().par_iter_mut()
  }
}
