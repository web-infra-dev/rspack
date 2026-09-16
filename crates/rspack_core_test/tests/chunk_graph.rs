use std::collections::{HashMap, HashSet};

use rspack_core::{BuildChunkGraphArtifact, ChunkGraph, ChunkKind, ChunkUkey};

fn create(graph: &mut ChunkGraph) -> ChunkUkey {
  graph
    .create_chunk(None, ChunkKind::Normal)
    .expect("allocate Chunk")
}

#[test]
fn recycled_slot_rejects_old_identity() {
  let mut artifact = BuildChunkGraphArtifact::default();
  let old = create(&mut artifact.chunk_graph);
  assert!(artifact.remove_chunk(&old).is_some());
  assert!(artifact.remove_chunk(&old).is_none());
  let current = create(&mut artifact.chunk_graph);
  assert_eq!(old.as_u64() as u32, current.as_u64() as u32);
  assert_ne!(old, current);
  assert!(artifact.chunk_graph.chunks.get(&old).is_none());
  assert!(artifact.chunk_graph.chunks.get_mut(&old).is_none());
  assert!(artifact.remove_chunk(&old).is_none());
  assert!(artifact.chunk_graph.chunks.contains(&current));
  assert_eq!(artifact.chunk_graph.chunks.len(), 1);
  let next = create(&mut artifact.chunk_graph);
  assert_ne!(next.as_u64() as u32, current.as_u64() as u32);
}

#[test]
fn snapshot_branches_cannot_alias_cached_values() {
  let mut artifact = BuildChunkGraphArtifact::default();
  let original = create(&mut artifact.chunk_graph);
  let snapshot = artifact.chunk_graph.clone();
  artifact.remove_chunk(&original);
  let previous = create(&mut artifact.chunk_graph);
  let cache = HashMap::from([(previous, "previous optimization")]);
  let history = HashSet::from([original, previous]);

  artifact.chunk_graph.clone_from(&snapshot);
  assert!(artifact.chunk_graph.chunks.contains(&original));
  artifact.remove_chunk(&original);
  let current = create(&mut artifact.chunk_graph);
  assert_eq!(previous.as_u64() as u32, current.as_u64() as u32);
  assert_ne!(previous, current);
  assert!(!cache.contains_key(&current));
  assert_eq!(history.len(), 2);
  assert!(artifact.remove_chunk(&previous).is_none());
  assert!(snapshot.chunks.contains(&original));
}

#[test]
fn independently_created_graphs_have_distinct_keys() {
  let mut a = ChunkGraph::default();
  let mut b = ChunkGraph::default();
  let a_key = create(&mut a);
  let b_key = create(&mut b);
  assert_eq!(a_key.as_u64() as u32, 0);
  assert_eq!(b_key.as_u64() as u32, 0);
  assert_ne!(a_key, b_key);
  assert_ne!(a_key.as_u32(), 0);
  assert_ne!(b_key.as_u32(), 0);
  assert!(a.chunks.get(&b_key).is_none());
  assert!(b.chunks.get_mut(&a_key).is_none());
  assert_eq!(std::mem::size_of::<ChunkUkey>(), 8);
  assert_eq!(ChunkUkey::from_u64(a_key.as_u64()), Some(a_key));
}

#[test]
fn slot_layout_uses_the_identity_niche() {
  fn check<T>() {
    assert_eq!(
      std::mem::size_of::<Option<(ChunkUkey, T)>>(),
      std::mem::size_of::<(ChunkUkey, T)>()
    );
  }
  check::<()>();
  check::<u64>();
  check::<[u8; 64]>();
  check::<rspack_core::Chunk>();
  check::<rspack_core::ChunkGraphChunk>();
  assert!(ChunkUkey::from_u64(0).is_none());
  assert!(ChunkUkey::from_u64(u32::MAX as u64).is_none());
}

#[test]
fn snapshots_preserve_free_slots_and_append_after_reuse() {
  let mut artifact = BuildChunkGraphArtifact::default();
  let original = (0..8)
    .map(|_| create(&mut artifact.chunk_graph))
    .collect::<Vec<_>>();
  for index in [2, 5, 0] {
    assert!(artifact.remove_chunk(&original[index]).is_some());
  }
  assert!(artifact.remove_chunk(&original[5]).is_none());
  let snapshot = artifact.chunk_graph.clone();
  let mut identities = HashSet::new();
  for _ in 0..3 {
    artifact.chunk_graph.clone_from(&snapshot);
    for index in [0, 5, 2, 8] {
      let key = create(&mut artifact.chunk_graph);
      assert_eq!(key.as_u64() as u32, index);
      assert_ne!(key.as_u32(), 0);
      assert!(identities.insert(key));
      assert_eq!(artifact.chunk_graph.get_number_of_chunk_modules(&key), 0);
      assert!(ChunkUkey::from_u64(index as u64).is_none());
    }
    assert_eq!(artifact.chunk_graph.chunks.len(), 9);
    assert_eq!(artifact.chunk_graph.chunks.values().count(), 9);
    for index in [2, 5, 0] {
      assert!(artifact.remove_chunk(&original[index]).is_none());
    }
  }
}

#[test]
fn removal_cleans_named_chunks_and_module_relationships() {
  let mut artifact = BuildChunkGraphArtifact::default();
  let (key, created) = artifact
    .add_named_chunk("entry".into())
    .expect("named Chunk");
  assert!(created);
  assert_eq!(
    artifact
      .add_named_chunk("entry".into())
      .expect("existing Chunk"),
    (key, false)
  );
  let module = rspack_collections::Identifier::from("module");
  artifact.chunk_graph.connect_chunk_and_module(key, module);
  artifact
    .chunk_graph
    .connect_chunk_and_runtime_module(key, module);
  artifact.remove_chunk(&key);
  assert!(artifact.named_chunks.is_empty());
  assert!(artifact.chunk_graph.get_module_chunks(module).is_empty());
  let (next, _) = artifact
    .add_named_chunk("entry".into())
    .expect("replacement Chunk");
  assert_ne!(key, next);
  assert_eq!(artifact.chunk_graph.get_number_of_chunk_modules(&next), 0);
}

#[test]
fn repeated_snapshot_restore_keeps_storage_bounded() {
  let mut artifact = BuildChunkGraphArtifact::default();
  let key = create(&mut artifact.chunk_graph);
  artifact.remove_chunk(&key);
  let snapshot = artifact.chunk_graph.clone();
  for _ in 0..1000 {
    artifact.chunk_graph.clone_from(&snapshot);
    let key = create(&mut artifact.chunk_graph);
    assert_eq!(key.as_u64() as u32, 0);
    assert_eq!(artifact.chunk_graph.chunks.values().count(), 1);
    artifact.remove_chunk(&key);
    assert_eq!(artifact.chunk_graph.chunks.keys().count(), 0);
  }
}

#[test]
fn diagnostics_cannot_persist_process_local_chunk_keys() {
  let mut diagnostic = rspack_error::Diagnostic::warn("test".into(), "message".into());
  assert!(rspack_cacheable::to_bytes(&diagnostic, &()).is_ok());
  let key = create(&mut ChunkGraph::default());
  diagnostic.chunk = Some(key.as_u64());
  assert!(matches!(
    rspack_cacheable::to_bytes(&diagnostic, &()),
    Err(rspack_cacheable::Error::UnsupportedField)
  ));
}

#[test]
fn removing_entry_chunk_cleans_both_sides_of_the_group() {
  let mut artifact = BuildChunkGraphArtifact::default();
  let key = create(&mut artifact.chunk_graph);
  let module = rspack_collections::Identifier::from("entry-module");
  let mut group = rspack_core::ChunkGroup::default();
  group.set_entrypoint_chunk(key);
  assert!(group.is_entrypoint_chunk(&key));
  let group_key = group.ukey();
  group.chunks.push(key);
  artifact
    .chunk_graph
    .chunks
    .expect_get_mut(&key)
    .add_group(group_key);
  artifact.chunk_group_by_ukey.add(group);
  artifact
    .chunk_graph
    .connect_chunk_and_entry_module(key, module, group_key);
  assert!(artifact.chunk_graph.is_entry_module(&module));
  artifact.remove_chunk(&key);
  assert!(!artifact.chunk_graph.is_entry_module(&module));
  assert!(
    !artifact
      .chunk_group_by_ukey
      .expect_get(&group_key)
      .is_entrypoint_chunk(&key)
  );
  assert!(
    artifact
      .chunk_group_by_ukey
      .expect_get(&group_key)
      .chunks
      .is_empty()
  );
}
