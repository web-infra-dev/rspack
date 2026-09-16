use std::{
  cell::Cell,
  panic::{AssertUnwindSafe, catch_unwind},
  rc::Rc,
};

use rspack_core::{BuildChunkGraphArtifact, ChunkKind, ChunkMap, ChunkSet};

#[test]
fn set_retains_slot_indices_and_rejects_stale_removal() {
  let mut artifact = BuildChunkGraphArtifact::default();
  let keys: Vec<_> = (0..8)
    .map(|_| {
      artifact
        .chunk_graph
        .create_chunk(None, ChunkKind::Normal)
        .unwrap()
    })
    .collect();
  let mut set = ChunkSet::with_capacity(artifact.chunk_graph.chunks.slot_count());
  for key in [keys[7], keys[0], keys[3]] {
    assert!(set.insert(&artifact.chunk_graph.chunks, key));
    assert!(!set.insert(&artifact.chunk_graph.chunks, key));
  }
  assert_eq!(set.len(), 3);
  set.retain(|key| *key != keys[3]);
  assert!(!set.contains(&keys[3]));
  assert!(set.contains(&keys[7]));
  assert_eq!(set.iter().count(), 2);

  artifact.remove_chunk(&keys[0]);
  let replacement = artifact
    .chunk_graph
    .create_chunk(None, ChunkKind::Normal)
    .unwrap();
  assert!(set.insert(&artifact.chunk_graph.chunks, replacement));
  assert!(!set.contains(&keys[0]));
  assert!(!set.remove(&keys[0]));
  assert!(set.contains(&replacement));
  assert_eq!(set.len(), 2);
  assert!(set.remove(&replacement));
  assert_eq!(set.into_keys().collect::<Vec<_>>(), vec![keys[7]]);
}

#[test]
fn sparse_insertion_reuse_and_restored_identity() {
  let mut artifact = BuildChunkGraphArtifact::default();
  let keys: Vec<_> = (0..32)
    .map(|_| {
      artifact
        .chunk_graph
        .create_chunk(None, ChunkKind::Normal)
        .unwrap()
    })
    .collect();
  let snapshot = artifact.chunk_graph.clone();
  let mut map = ChunkMap::with_capacity(artifact.chunk_graph.chunks.slot_count());
  assert_eq!(map.slot_count(), 0); // Reserving does not initialize slots.
  assert!(map.is_empty());
  assert_eq!(map.insert(&artifact.chunk_graph.chunks, keys[31], 31), None);
  assert_eq!(map.slot_count(), 32);
  assert_eq!(map.len(), 1);
  assert_eq!(map.get(&keys[0]), None);
  map.insert(&artifact.chunk_graph.chunks, keys[0], 0);
  assert_eq!(
    map.insert(&artifact.chunk_graph.chunks, keys[31], 32),
    Some(31)
  );
  assert_eq!(
    map.keys().copied().collect::<Vec<_>>(),
    vec![keys[0], keys[31]]
  );

  artifact.remove_chunk(&keys[0]);
  let replacement = artifact
    .chunk_graph
    .create_chunk(None, ChunkKind::Normal)
    .unwrap();
  assert_eq!(
    map.insert(&artifact.chunk_graph.chunks, replacement, 100),
    None
  );
  assert_eq!(map.get(&keys[0]), None);
  assert_eq!(map.remove(&keys[0]), None);
  assert_eq!(map.len(), 2);
  // Membership, not identity ordering, decides whether restored keys are valid.
  artifact.chunk_graph = snapshot;
  assert_eq!(map.insert(&artifact.chunk_graph.chunks, keys[0], 200), None);
  assert_eq!(map.get(&replacement), None);
  assert_eq!(map.get(&keys[0]), Some(&200));
  assert_eq!(map.len(), 2);
  artifact.remove_chunk(&keys[0]);
  assert_eq!(map.remove(&keys[0]), Some(200));
  assert_eq!(map.remove(&keys[0]), None);
}

#[test]
fn stale_and_foreign_insertions_cannot_evict_live_values() {
  let mut artifact = BuildChunkGraphArtifact::default();
  let old = artifact
    .chunk_graph
    .create_chunk(None, ChunkKind::Normal)
    .unwrap();
  artifact.remove_chunk(&old);
  let live = artifact
    .chunk_graph
    .create_chunk(None, ChunkKind::Normal)
    .unwrap();
  let mut other = rspack_core::ChunkGraph::default();
  let foreign = other.create_chunk(None, ChunkKind::Normal).unwrap();
  let mut map = ChunkMap::with_capacity(1);
  map.insert(&artifact.chunk_graph.chunks, live, 42);
  for invalid in [old, foreign] {
    assert!(
      catch_unwind(AssertUnwindSafe(|| {
        map.insert(&artifact.chunk_graph.chunks, invalid, 0);
      }))
      .is_err()
    );
    assert!(
      catch_unwind(AssertUnwindSafe(|| {
        map.get_or_insert_default(&artifact.chunk_graph.chunks, invalid);
      }))
      .is_err()
    );
    assert_eq!(map.get(&live), Some(&42));
    assert_eq!(map.len(), 1);
  }
}

#[test]
fn payloads_without_default_are_dropped_exactly_once() {
  #[derive(Clone)]
  struct Payload(Rc<Cell<usize>>);
  impl Drop for Payload {
    fn drop(&mut self) {
      self.0.set(self.0.get() + 1);
    }
  }
  let drops = Rc::new(Cell::new(0));
  let mut graph = rspack_core::ChunkGraph::default();
  let key = graph.create_chunk(None, ChunkKind::Normal).unwrap();
  let mut map = ChunkMap::with_capacity(16);
  map.insert(&graph.chunks, key, Payload(drops.clone()));
  let snapshot = (*map).clone();
  let removed = map.remove(&key).unwrap();
  assert_eq!(drops.get(), 0);
  drop(removed);
  assert_eq!(drops.get(), 1);
  map.insert(&graph.chunks, key, Payload(drops.clone()));
  drop(map);
  assert_eq!(drops.get(), 2);
  drop(snapshot);
  assert_eq!(drops.get(), 3);
}
