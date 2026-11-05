#![cfg(unix)]
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use rspack_core::{
  BoxModule, BuildModuleGraphArtifact, CompilationLogger, Module, ModuleGraphModule,
  ModuleIdentifier, PortableRoot, RawModule, RuntimeGlobals, SnapshotOptions,
  SnapshotStrategyOptions,
  cache::CacheCodec,
  legacy_cache::persistent::{
    build_dependencies::BuildDeps,
    context::CacheContext,
    occasion::{MakeOccasion, Occasion},
    snapshot::{Snapshot, SnapshotScope, Strategy},
    validation::CacheValidation,
  },
};
use rspack_fs::{MemoryFileSystem, NativeFileSystem, ReadableFileSystem, WritableFileSystem};
use rspack_paths::{AssertUtf8, InternedPath};
use rspack_storage::{MemoryStorage, Storage};

fn codec(root: &str, canonical: &str) -> Arc<CacheCodec> {
  Arc::new(CacheCodec::new(Some(PortableRoot {
    root: root.into(),
    canonical_root: canonical.into(),
  })))
}

fn options() -> SnapshotOptions {
  SnapshotOptions {
    immutable_paths: vec![],
    unmanaged_paths: vec![],
    managed_paths: vec![],
    resolve_build_dependencies: SnapshotStrategyOptions::hash(),
    build_dependencies: SnapshotStrategyOptions::hash(),
    resolve: SnapshotStrategyOptions::hash(),
    module: SnapshotStrategyOptions::hash(),
    context_module: SnapshotStrategyOptions::hash(),
  }
}

#[tokio::test]
async fn expired_layout_files_compare_hashes_when_timestamps_match() {
  use std::{
    fs,
    time::{Duration, SystemTime},
  };

  let temp = tempfile::tempdir().unwrap();
  let base = fs::canonicalize(temp.path()).unwrap();
  // Future mtime dominates ctime on Unix, making equality deterministic across roots.
  let time = SystemTime::now() + Duration::from_secs(3600);
  for (name, contents) in [("writer", "seven"), ("same", "seven"), ("changed", "other")] {
    let root = base.join(name);
    fs::create_dir_all(&root).unwrap();
    let path = root.join("input.js");
    fs::write(&path, contents).unwrap();
    fs::File::options()
      .write(true)
      .open(path)
      .unwrap()
      .set_times(fs::FileTimes::new().set_modified(time))
      .unwrap();
  }
  let writer = base.join("writer").assert_utf8();
  let filesystem = Arc::new(NativeFileSystem::new(false));
  let writer_meta = filesystem
    .metadata(writer.join("input.js").as_path())
    .await
    .unwrap();
  let mut storage = MemoryStorage::default();
  Snapshot::new(
    options(),
    writer.as_str().into(),
    filesystem.clone(),
    codec_for_moved(writer.as_str()),
  )
  .add(
    &mut storage,
    SnapshotScope::FILE,
    [InternedPath::from(writer.join("input.js").as_std_path()).into_fixed()].into_iter(),
  )
  .await;
  let entries = storage.load(SnapshotScope::FILE.name()).await.unwrap();
  assert_eq!(entries.len(), 2);
  for (_, value) in &entries {
    let (bound_root, strategy) = codec_for_moved(writer.as_str())
      .decode::<(Option<u64>, Strategy)>(value)
      .unwrap();
    assert!(bound_root.is_some());
    assert!(matches!(strategy, Strategy::FileTimestampAndHash { .. }));
  }
  for (name, changed) in [("same", false), ("changed", true)] {
    let root = base.join(name).assert_utf8();
    let path = root.join("input.js");
    let meta = filesystem.metadata(&path).await.unwrap();
    assert_eq!(
      writer_meta.mtime_ms.max(writer_meta.ctime_ms),
      meta.mtime_ms.max(meta.ctime_ms)
    );
    let reader = Snapshot::new(
      options(),
      root.as_str().into(),
      filesystem.clone(),
      codec_for_moved(root.as_str()),
    );
    let (_, modified, deleted, unchanged, _, _) = reader
      .calc_modified_paths(&storage, SnapshotScope::FILE)
      .await
      .unwrap();
    let path = InternedPath::from(path.as_std_path());
    assert_eq!(modified.contains(&path), changed);
    assert_eq!(unchanged.contains(&path), !changed);
    assert!(!deleted.contains(&path));
  }
}

/// Records the cache as written under `root`, so a reader elsewhere sees a relocated cache.
fn save_meta(storage: &mut MemoryStorage, root: &str, codec: Arc<CacheCodec>) {
  let filesystem: Arc<dyn ReadableFileSystem> = Arc::new(NativeFileSystem::new(false));
  let snapshot = Snapshot::new(options(), root.into(), filesystem.clone(), codec.clone());
  // Saving records the current dependency id counter of the compiler context.
  rspack_tasks::within_compiler_context_for_testing_sync(|| {
    CacheValidation::new(
      codec,
      "test".into(),
      BuildDeps::new(&vec![], filesystem, Arc::new(snapshot)),
    )
    .save(storage);
  });
}

/// Sets a future mtime, which dominates ctime on Unix: copies at other roots then share the
/// snapshot's modified time while their content is free to differ.
fn set_future_mtime(path: &std::path::Path, time: std::time::SystemTime) {
  use std::fs;
  fs::File::options()
    .write(true)
    .open(path)
    .unwrap()
    .set_times(fs::FileTimes::new().set_modified(time))
    .unwrap();
}

#[tokio::test]
async fn relocated_ordinary_files_compare_hashes_when_timestamps_match() {
  use std::fs;

  let temp = tempfile::tempdir().unwrap();
  let base = fs::canonicalize(temp.path()).unwrap();
  let time = std::time::SystemTime::now() + std::time::Duration::from_secs(3600);
  for (name, contents) in [("writer", "OLD"), ("same", "OLD"), ("changed", "NEW")] {
    let root = base.join(name);
    fs::create_dir_all(&root).unwrap();
    let path = root.join("dep.js");
    fs::write(&path, contents).unwrap();
    set_future_mtime(&path, time);
  }
  let writer = base.join("writer").assert_utf8();
  let filesystem = Arc::new(NativeFileSystem::new(false));
  let mut storage = MemoryStorage::default();
  let writer_codec = codec_for_moved(writer.as_str());
  let writer_snapshot = Snapshot::new(
    options(),
    writer.as_str().into(),
    filesystem.clone(),
    writer_codec.clone(),
  );
  // An ordinary input, as a module's own file is recorded: no root binding.
  writer_snapshot
    .add(
      &mut storage,
      SnapshotScope::FILE,
      [InternedPath::from(writer.join("dep.js").as_std_path())].into_iter(),
    )
    .await;
  save_meta(&mut storage, writer.as_str(), writer_codec.clone());
  let entries = storage.load(SnapshotScope::FILE.name()).await.unwrap();
  assert_eq!(entries.len(), 1);
  let (bound_root, strategy) = writer_codec
    .decode::<(Option<u64>, Strategy)>(&entries[0].1)
    .unwrap();
  assert!(bound_root.is_none());
  let Strategy::FileTimestampAndHash { mtime, .. } = strategy else {
    panic!("expected a timestamp and hash record");
  };

  for (name, changed) in [("same", false), ("changed", true)] {
    let root = base.join(name).assert_utf8();
    let path = root.join("dep.js");
    let meta = filesystem.metadata(&path).await.unwrap();
    assert_eq!(mtime, meta.mtime_ms.max(meta.ctime_ms));
    let reader = Snapshot::new(
      options(),
      root.as_str().into(),
      filesystem.clone(),
      codec_for_moved(root.as_str()),
    );
    let (_, modified, deleted, unchanged, observed, _) = reader
      .calc_modified_paths(&storage, SnapshotScope::FILE)
      .await
      .unwrap();
    let path = InternedPath::from(path.as_std_path());
    assert_eq!(modified.contains(&path), changed, "{name}");
    assert_eq!(unchanged.contains(&path), !changed, "{name}");
    assert!(!deleted.contains(&path));
    // An unchanged record is re-stored with this root's observation.
    assert_eq!(observed.contains_key(&path), !changed, "{name}");
  }

  // The same cache under its own root still takes the timestamp shortcut.
  let (_, modified, _, unchanged, observed, _) = writer_snapshot
    .calc_modified_paths(&storage, SnapshotScope::FILE)
    .await
    .unwrap();
  let path = InternedPath::from(writer.join("dep.js").as_std_path());
  assert!(modified.is_empty());
  assert!(unchanged.contains(&path));
  assert!(observed.is_empty());
}

#[tokio::test]
async fn relocated_file_replaced_by_link_to_the_writer_file_is_modified() {
  use std::{fs, os::unix::fs::symlink};

  let temp = tempfile::tempdir().unwrap();
  let base = fs::canonicalize(temp.path()).unwrap();
  fs::create_dir_all(base.join("writer")).unwrap();
  fs::create_dir_all(base.join("reader")).unwrap();
  fs::write(
    base.join("writer/bridge.js"),
    "export { value } from './value';",
  )
  .unwrap();
  let writer = base.join("writer").assert_utf8();
  let filesystem = Arc::new(NativeFileSystem::new(false));
  let mut storage = MemoryStorage::default();
  let writer_codec = codec_for_moved(writer.as_str());
  let writer_snapshot = Snapshot::new(
    options(),
    writer.as_str().into(),
    filesystem.clone(),
    writer_codec.clone(),
  );
  writer_snapshot
    .add(
      &mut storage,
      SnapshotScope::FILE,
      [InternedPath::from(writer.join("bridge.js").as_std_path())].into_iter(),
    )
    .await;
  save_meta(&mut storage, writer.as_str(), writer_codec);

  // The reader's file is a link into the writer checkout: following it reports the writer
  // file's timestamps, yet imports now resolve relative to the writer.
  symlink(base.join("writer/bridge.js"), base.join("reader/bridge.js")).unwrap();
  let reader = base.join("reader").assert_utf8();
  let reader_snapshot = Snapshot::new(
    options(),
    reader.as_str().into(),
    filesystem.clone(),
    codec_for_moved(reader.as_str()),
  );
  let (_, modified, deleted, unchanged, _, _) = reader_snapshot
    .calc_modified_paths(&storage, SnapshotScope::FILE)
    .await
    .unwrap();
  let path = InternedPath::from(reader.join("bridge.js").as_std_path());
  assert!(modified.contains(&path));
  assert!(!unchanged.contains(&path));
  assert!(!deleted.contains(&path));
}

#[tokio::test]
async fn snapshot_first_use_pads_timestamp_precision() {
  let fs = Arc::new(MemoryFileSystem::default());
  fs.create_dir_all("/project/dir".into()).await.unwrap();
  let metadata = fs.metadata("/project/dir".into()).await.unwrap();
  let newest = metadata.ctime_ms.max(metadata.mtime_ms);
  let codec = codec("/project", "/project");
  let snapshot = Snapshot::new(options(), "/project".into(), fs, codec.clone());
  let mut storage = MemoryStorage::default();
  snapshot
    .add_unobserved_contexts(
      &mut storage,
      [InternedPath::from("/project/dir")].into_iter(),
      Some(newest + 1),
    )
    .await;
  let entries = storage.load(SnapshotScope::CONTEXT.name()).await.unwrap();
  assert_eq!(entries.len(), 1);
  assert!(matches!(
    codec
      .decode::<(Option<u64>, Strategy)>(&entries[0].1)
      .unwrap()
      .1,
    Strategy::Failed
  ));
}

fn artifact(identifier: ModuleIdentifier, source: &str) -> BuildModuleGraphArtifact {
  let mut item = BuildModuleGraphArtifact::new();
  item
    .module_graph
    .add_module_graph_module(ModuleGraphModule::new(identifier));
  let module = RawModule::new(
    source.into(),
    identifier,
    "module".into(),
    RuntimeGlobals::empty(),
  );
  module.freeze_build_info();
  module.freeze_build_meta();
  item
    .module_graph
    .add_module(BoxModule::new(Box::new(module)));
  item.issuer_update_modules.insert(identifier);
  item
}

#[tokio::test]
async fn module_keys_drop_all_aliases_and_never_resurrect_after_mutation() {
  rspack_tasks::within_compiler_context_for_testing(async {
    let writer = codec("/alias/project", "/real/project");
    let reader = codec("/moved/project", "/moved/project");
    let old_identifier = ModuleIdentifier::from("/alias/project/module.js");
    let identifier = ModuleIdentifier::from("/moved/project/module.js");
    let mut storage = MemoryStorage::default();
    MakeOccasion::new(writer.clone()).save(&mut storage, &artifact(old_identifier, "old"));
    let occasion = MakeOccasion::new(reader.clone());
    let logger = CompilationLogger::new("test", Default::default());
    let old_key = writer.encode(&old_identifier).unwrap();
    let new_key = reader.encode(&identifier).unwrap();
    assert_ne!(old_key, new_key);
    let scope = occasion.scope();
    let recovered = occasion.recovery(&storage, &logger).await.unwrap();
    assert_eq!(recovered.item.module_graph.modules_len(), 0);
    assert_eq!(recovered.dropped, vec![old_key.clone()]);
    for key in recovered.dropped {
      storage.remove(scope, &key);
    }
    occasion.save(&mut storage, &artifact(identifier, "mutated"));
    for _ in 0..2 {
      let recovered = occasion.recovery(&storage, &logger).await.unwrap();
      assert!(recovered.dropped.is_empty());
      assert_eq!(recovered.item.module_graph.modules_len(), 1);
      assert_eq!(storage.load(scope).await.unwrap().len(), 1);
      assert_eq!(
        recovered
          .item
          .module_graph
          .module_by_identifier(&identifier)
          .unwrap()
          .size(None),
        7.0
      );
    }
  })
  .await;
}

#[tokio::test]
async fn module_keys_drop_all_duplicate_decodings() {
  rspack_tasks::within_compiler_context_for_testing(async {
    let writer = codec("/alias/project", "/real/project");
    let reader = codec("/moved/project", "/moved/project");
    let mut storage = MemoryStorage::default();
    let identifier = ModuleIdentifier::from("/moved/project/module.js");
    MakeOccasion::new(writer).save(
      &mut storage,
      &artifact("/alias/project/module.js".into(), "old"),
    );
    let occasion = MakeOccasion::new(reader);
    occasion.save(&mut storage, &artifact(identifier, "mutated"));
    let scope = occasion.scope();
    let keys = storage
      .load(scope)
      .await
      .unwrap()
      .into_iter()
      .map(|(key, _)| key)
      .collect::<Vec<_>>();
    assert_eq!(keys.len(), 2);
    let recovered = occasion
      .recovery(
        &storage,
        &CompilationLogger::new("test", Default::default()),
      )
      .await
      .unwrap();
    assert_eq!(recovered.item.module_graph.modules_len(), 0);
    assert_eq!(recovered.dropped.len(), 2);
    for key in keys {
      assert!(recovered.dropped.contains(&key));
    }
    for key in recovered.dropped {
      storage.remove(scope, &key);
    }
    assert!(storage.load(scope).await.unwrap().is_empty());
  })
  .await;
}

#[tokio::test]
async fn snapshot_keys_fail_closed_after_relocation_and_mutation() {
  let fs = Arc::new(MemoryFileSystem::default());
  fs.create_dir_all("/moved/project".into()).await.unwrap();
  fs.write("/moved/project/module.js".into(), b"old")
    .await
    .unwrap();
  let writer = codec("/alias/project", "/real/project");
  let reader = codec("/moved/project", "/moved/project");
  let path = InternedPath::from("/moved/project/module.js");
  let old_key = writer
    .encode(&InternedPath::from("/alias/project/module.js"))
    .unwrap();
  let snapshot = Snapshot::new(
    options(),
    "/moved/project".into(),
    fs.clone(),
    reader.clone(),
  );
  let mut storage = MemoryStorage::default();
  let scope = SnapshotScope::FILE;
  snapshot
    .add(&mut storage, scope, [path.clone()].into_iter())
    .await;
  let new_key = reader.encode(&path).unwrap();
  let value = storage.load(scope.name()).await.unwrap()[0].1.clone();
  storage.remove(scope.name(), &new_key);
  storage.set(scope.name(), old_key.clone(), value.clone());
  for readonly in [false, true] {
    let mut stored = MemoryStorage::default();
    stored.set(scope.name(), old_key.clone(), value.clone());
    let mut context = CacheContext::new(
      Box::new(stored),
      readonly,
      CompilationLogger::new("test", Default::default()),
    );
    let (_, modified, _) = context.load_snapshot(&snapshot).await.unwrap();
    assert!(modified.contains(&path));
    let (hot, modified, _) = context.load_snapshot(&snapshot).await.unwrap();
    assert_eq!(
      hot, readonly,
      "only writable recovery evicts the original key"
    );
    assert_eq!(modified.contains(&path), readonly);
  }
  let (_, modified, _, _, _, dropped) =
    snapshot.calc_modified_paths(&storage, scope).await.unwrap();
  assert!(modified.contains(&path));
  assert_eq!(dropped, vec![old_key]);
  for key in dropped {
    storage.remove(scope.name(), &key);
  }
  fs.write("/moved/project/module.js".into(), b"mutated")
    .await
    .unwrap();
  snapshot
    .add(&mut storage, scope, [path.clone()].into_iter())
    .await;
  for _ in 0..2 {
    let (_, modified, deleted, unchanged, _, dropped) =
      snapshot.calc_modified_paths(&storage, scope).await.unwrap();
    assert!(modified.is_empty() && deleted.is_empty() && dropped.is_empty());
    assert!(unchanged.contains(&path));
    assert_eq!(storage.load(scope.name()).await.unwrap().len(), 1);
  }
}

async fn check_symlink_snapshots(indirect: bool) {
  use std::{fs, os::unix::fs::symlink};
  let temp = tempfile::tempdir().unwrap();
  let root = temp.path().join("writer");
  fs::create_dir_all(root.join("scan")).unwrap();
  fs::write(root.join("a.js"), "same").unwrap();
  fs::write(root.join("b.js"), "same").unwrap();
  symlink("../a.js", root.join("scan/item.js")).unwrap();
  symlink("a.js", root.join("pivot.js")).unwrap();
  symlink("../pivot.js", root.join("scan/indirect.js")).unwrap();
  let root = root.assert_utf8();
  let canonical = std::fs::canonicalize(&root).unwrap().assert_utf8();
  let codec = codec(root.as_str(), canonical.as_str());
  let fs = Arc::new(NativeFileSystem::new(false));
  let snapshot = Snapshot::new(options(), root.as_str().into(), fs.clone(), codec);
  let path = InternedPath::from(root.join("scan").as_std_path());
  let mut storage = MemoryStorage::default();
  snapshot
    .add(
      &mut storage,
      SnapshotScope::CONTEXT,
      [path.clone()].into_iter(),
    )
    .await;
  let link = if indirect {
    root.join("pivot.js")
  } else {
    root.join("scan/item.js")
  };
  std::fs::remove_file(&link).unwrap();
  symlink(if indirect { "b.js" } else { "../b.js" }, &link).unwrap();
  let (_, modified, _, _, _, _) = snapshot
    .calc_modified_paths(&storage, SnapshotScope::CONTEXT)
    .await
    .unwrap();
  assert!(
    modified.contains(&path),
    "equal-content retarget must invalidate (indirect={indirect})"
  );
  snapshot
    .add(&mut storage, SnapshotScope::CONTEXT, [path].into_iter())
    .await;
  let moved = temp.path().join("reader").assert_utf8();
  std::fs::rename(&root, &moved).unwrap();
  let reader = codec_for_moved(moved.as_str());
  let snapshot = Snapshot::new(options(), moved.as_str().into(), fs, reader);
  let (_, modified, deleted, unchanged, _, dropped) = snapshot
    .calc_modified_paths(&storage, SnapshotScope::CONTEXT)
    .await
    .unwrap();
  assert!(modified.is_empty() && deleted.is_empty() && dropped.is_empty());
  assert!(unchanged.contains(&InternedPath::from(moved.join("scan").as_std_path())));
}

#[cfg(unix)]
fn codec_for_moved(root: &str) -> Arc<CacheCodec> {
  let canonical = std::fs::canonicalize(root).unwrap().assert_utf8();
  codec(root, canonical.as_str())
}

#[tokio::test]
async fn symlink_snapshot_direct_equal_content_retarget_and_relocation() {
  check_symlink_snapshots(false).await;
}

#[tokio::test]
async fn symlink_snapshot_intermediate_equal_content_retarget_and_relocation() {
  check_symlink_snapshots(true).await;
}

#[tokio::test]
async fn symlink_snapshot_errors_fail_closed_including_managed_roots() {
  use std::{fs, os::unix::fs::symlink};

  use rspack_core::PathMatcher;
  let temp = tempfile::tempdir().unwrap();
  // A canonical root keeps the walk identical on hosts whose temp dir is behind a link.
  let root = fs::canonicalize(temp.path()).unwrap().assert_utf8();
  let root = root.as_path();
  fs::create_dir(root.join("scan")).unwrap();
  symlink(".", root.join("scan/loop")).unwrap();
  let mut configuration = options();
  configuration
    .managed_paths
    .push(PathMatcher::String(root.join("scan").into_string()));
  let snapshot = Snapshot::new(
    configuration,
    root.as_str().into(),
    Arc::new(NativeFileSystem::new(false)),
    codec_for_moved(root.as_str()),
  );
  let mut storage = MemoryStorage::default();
  let path = InternedPath::from(root.join("scan").as_std_path());
  snapshot
    .add(
      &mut storage,
      SnapshotScope::CONTEXT,
      [path.clone()].into_iter(),
    )
    .await;
  let entries = storage.load(SnapshotScope::CONTEXT.name()).await.unwrap();
  assert_eq!(entries.len(), 1);
  let decoder = codec_for_moved(root.as_str());
  assert!(matches!(
    decoder
      .decode::<(Option<u64>, Strategy)>(&entries[0].1)
      .unwrap()
      .1,
    Strategy::Failed
  ));
  fs::remove_file(root.join("scan/loop")).unwrap();
  symlink("missing", root.join("scan/broken")).unwrap();
  snapshot
    .add(&mut storage, SnapshotScope::CONTEXT, [path].into_iter())
    .await;
  let entries = storage.load(SnapshotScope::CONTEXT.name()).await.unwrap();
  assert!(matches!(
    decoder
      .decode::<(Option<u64>, Strategy)>(&entries[0].1)
      .unwrap()
      .1,
    Strategy::Failed
  ));
  // The snapshot root itself is a link under managedPaths, so package shortcuts must not hide it.
  let path = InternedPath::from(root.join("scan/broken").as_std_path());
  snapshot
    .add(&mut storage, SnapshotScope::FILE, [path].into_iter())
    .await;
  let entries = storage.load(SnapshotScope::FILE.name()).await.unwrap();
  assert!(matches!(
    decoder
      .decode::<(Option<u64>, Strategy)>(&entries[0].1)
      .unwrap()
      .1,
    Strategy::Failed
  ));
}

#[tokio::test]
async fn linked_file_dependencies_check_the_target_without_rehashing_it() {
  use std::{fs, os::unix::fs::symlink};
  let temp = tempfile::tempdir().unwrap();
  let root = fs::canonicalize(temp.path()).unwrap().join("writer");
  fs::create_dir_all(root.join("packages/a")).unwrap();
  fs::create_dir_all(root.join("packages/b")).unwrap();
  fs::create_dir_all(root.join("app/node_modules")).unwrap();
  for package in ["a", "b"] {
    fs::write(root.join("packages").join(package).join("index.js"), "same").unwrap();
  }
  symlink("../../packages/a", root.join("app/node_modules/dep")).unwrap();
  let root = root.assert_utf8();
  let snapshot = Snapshot::new(
    options(),
    root.as_str().into(),
    Arc::new(NativeFileSystem::new(false)),
    codec(root.as_str(), root.as_str()),
  );
  let link = InternedPath::from(root.join("app/node_modules/dep").as_std_path());
  let mut storage = MemoryStorage::default();
  snapshot
    .add(
      &mut storage,
      SnapshotScope::FILE,
      [link.clone()].into_iter(),
    )
    .await;
  let modified = |snapshot: Snapshot, storage: MemoryStorage| async move {
    let (_, modified, deleted, _, _, _) = snapshot
      .calc_modified_paths(&storage, SnapshotScope::FILE)
      .await
      .unwrap();
    (modified, deleted, snapshot, storage)
  };

  // Editing a file inside the linked package is left to that file's own snapshot.
  fs::write(root.join("packages/a/index.js"), "edited").unwrap();
  let (changed, deleted, snapshot, storage) = modified(snapshot, storage).await;
  assert!(changed.is_empty() && deleted.is_empty());

  // Adding an entry, or pointing the link at an equal package, invalidates the link.
  fs::write(root.join("packages/a/extra.js"), "").unwrap();
  let (changed, _, snapshot, storage) = modified(snapshot, storage).await;
  assert!(changed.contains(&link));
  fs::remove_file(root.join("packages/a/extra.js")).unwrap();
  let (changed, _, snapshot, storage) = modified(snapshot, storage).await;
  assert!(changed.is_empty());
  fs::remove_file(root.join("app/node_modules/dep")).unwrap();
  symlink("../../packages/b", root.join("app/node_modules/dep")).unwrap();
  let (changed, _, snapshot, mut storage) = modified(snapshot, storage).await;
  assert!(changed.contains(&link));

  // An unchanged in-root link survives relocating the root.
  snapshot
    .add(&mut storage, SnapshotScope::FILE, [link].into_iter())
    .await;
  let moved = temp.path().join("reader").assert_utf8();
  fs::rename(root, &moved).unwrap();
  let reader = Snapshot::new(
    options(),
    moved.as_str().into(),
    Arc::new(NativeFileSystem::new(false)),
    codec_for_moved(moved.as_str()),
  );
  let (changed, deleted, _, _) = modified(reader, storage).await;
  assert!(changed.is_empty() && deleted.is_empty());
}

#[tokio::test]
async fn deeper_root_invalidates_missing_ancestor_lookups() {
  let fs = Arc::new(MemoryFileSystem::default());
  let mut storage = MemoryStorage::default();
  let writer = Snapshot::new(
    options(),
    "/a/proj".into(),
    fs.clone(),
    codec("/a/proj", "/a/proj"),
  );
  writer
    .add(
      &mut storage,
      SnapshotScope::MISSING,
      ["/a/node_modules/optional", "/node_modules/optional"]
        .map(InternedPath::from)
        .into_iter(),
    )
    .await;
  // This directory is a new ancestor of the reader and has no writer-side counterpart.
  fs.create_dir_all("/b/node_modules/optional".into())
    .await
    .unwrap();
  let reader = Snapshot::new(
    options(),
    "/b/c/proj".into(),
    fs,
    codec("/b/c/proj", "/b/c/proj"),
  );
  let (_, modified, deleted, _, _, _) = reader
    .calc_modified_paths(&storage, SnapshotScope::MISSING)
    .await
    .unwrap();
  assert!(
    !modified.is_empty() || !deleted.is_empty(),
    "new ancestors must force resolution to run again"
  );
}

#[tokio::test]
async fn fixed_snapshot_keys_expire_after_relocation_and_remove_both_policies() {
  for fixed_path in ["/a/proj/optional.js", "/a/optional.js"] {
    let fs = Arc::new(MemoryFileSystem::default());
    let mut storage = MemoryStorage::default();
    let writer = Snapshot::new(
      options(),
      "/a/proj".into(),
      fs.clone(),
      codec("/a/proj", "/a/proj"),
    );
    let fixed = InternedPath::from(fixed_path).into_fixed();
    writer
      .add(
        &mut storage,
        SnapshotScope::MISSING,
        [fixed.clone()].into_iter(),
      )
      .await;
    // The second key covers a movable factorization sharing this same filesystem identity.
    assert_eq!(
      storage
        .load(SnapshotScope::MISSING.name())
        .await
        .unwrap()
        .len(),
      2
    );
    fs.create_dir_all("/a/proj".into()).await.unwrap();
    fs.write(fixed_path.into(), b"appeared").await.unwrap();
    let reader = Snapshot::new(options(), "/b/proj".into(), fs, codec("/b/proj", "/b/proj"));
    let (_, modified, _, _, _, dropped) = reader
      .calc_modified_paths(&storage, SnapshotScope::MISSING)
      .await
      .unwrap();
    assert!(modified.contains(&fixed));
    assert_eq!(dropped.len(), 2, "both fixed and layout records expire");
    // Removal cannot depend on which policy survived aggregation in the path counter.
    writer.remove(
      &mut storage,
      SnapshotScope::MISSING,
      [fixed.without_fixed()].into_iter(),
    );
    assert!(
      storage
        .load(SnapshotScope::MISSING.name())
        .await
        .unwrap()
        .is_empty()
    );
  }
}

#[tokio::test]
async fn mixed_snapshot_owners_validate_the_reader_in_both_registration_orders() {
  use rspack_core::DependencyId;
  use rspack_paths::InternedPathList;

  for fixed_first in [true, false] {
    let fs = Arc::new(MemoryFileSystem::default());
    for root in ["/a/project", "/b/project"] {
      fs.create_dir_all(root.into()).await.unwrap();
      fs.write(format!("{root}/shared.js").as_str().into(), b"same")
        .await
        .unwrap();
    }
    let file = InternedPath::from("/a/project/shared.js");
    let fixed = InternedPathList::from_vec(vec![file.clone().into_fixed()]);
    let relative = InternedPathList::from_vec(vec![file.clone()]);
    let mut artifact = BuildModuleGraphArtifact::new();
    let counter = &mut artifact.file_dependencies;
    if fixed_first {
      counter.add_dependency_files(DependencyId::from(1), &fixed);
    }
    counter.add_module_files("module".into(), [&file]);
    counter.add_dependency_files(DependencyId::from(2), &relative);
    if !fixed_first {
      counter.add_dependency_files(DependencyId::from(1), &fixed);
    }
    let mut storage = MemoryStorage::default();
    let writer = Snapshot::new(
      options(),
      "/a/project".into(),
      fs.clone(),
      codec("/a/project", "/a/project"),
    );
    writer
      .add(&mut storage, SnapshotScope::FILE, counter.files().cloned())
      .await;
    let reader = Snapshot::new(
      options(),
      "/b/project".into(),
      fs.clone(),
      codec("/b/project", "/b/project"),
    );
    let moved = InternedPath::from("/b/project/shared.js");
    let (_, modified, deleted, unchanged, _, dropped) = reader
      .calc_modified_paths(&storage, SnapshotScope::FILE)
      .await
      .unwrap();
    assert!(!modified.contains(&moved), "fixed_first={fixed_first}");
    assert!(!deleted.contains(&moved));
    assert!(unchanged.contains(&moved));
    if fixed_first {
      assert!(
        modified.contains(&file),
        "the raw fixed record still expires"
      );
      assert_eq!(dropped.len(), 2, "both encodings still expire");
    }
    // MemoryFileSystem timestamps have millisecond precision. Advance past the unchanged
    // file's timestamp so this tests content invalidation rather than an undetectable write.
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs.write("/b/project/shared.js".into(), b"changed reader")
      .await
      .unwrap();
    let (_, modified, _, _, _, _) = reader
      .calc_modified_paths(&storage, SnapshotScope::FILE)
      .await
      .unwrap();
    assert!(modified.contains(&moved), "fixed_first={fixed_first}");
  }
}

#[tokio::test]
async fn expired_layout_observations_survive_partial_saves_in_every_input_scope() {
  for scope in [
    SnapshotScope::FILE,
    SnapshotScope::CONTEXT,
    SnapshotScope::MISSING,
  ] {
    let fs = Arc::new(MemoryFileSystem::default());
    for root in ["/a/project", "/b/project"] {
      fs.create_dir_all(format!("{root}/directory").as_str().into())
        .await
        .unwrap();
      fs.write(format!("{root}/file").as_str().into(), b"same")
        .await
        .unwrap();
    }
    let name = match scope {
      SnapshotScope::FILE => "file",
      SnapshotScope::CONTEXT => "directory",
      SnapshotScope::MISSING => "missing",
      SnapshotScope::BUILD => unreachable!(),
    };
    let original = InternedPath::from(format!("/a/project/{name}").as_str()).into_fixed();
    let moved = InternedPath::from(format!("/b/project/{name}").as_str());
    let mut storage = MemoryStorage::default();
    Snapshot::new(
      options(),
      "/a/project".into(),
      fs.clone(),
      codec("/a/project", "/a/project"),
    )
    .add(&mut storage, scope, [original.clone()].into_iter())
    .await;
    let reader = Snapshot::new(
      options(),
      "/b/project".into(),
      fs.clone(),
      codec("/b/project", "/b/project"),
    );
    let mut context = CacheContext::new(
      Box::new(storage),
      false,
      CompilationLogger::new("test", Default::default()),
    );
    let (_, modified, deleted) = context.load_snapshot(&reader).await.unwrap();
    assert!(modified.contains(&original));
    assert!(!modified.contains(&moved) && !deleted.contains(&moved));
    let empty = || (std::iter::empty(), std::iter::empty());
    context
      .save_snapshot(&reader, empty(), empty(), empty())
      .await;
    let (hot, modified, deleted) = context.load_snapshot(&reader).await.unwrap();
    assert!(hot && modified.is_empty() && deleted.is_empty());
    let changed = match scope {
      SnapshotScope::CONTEXT => "/b/project/directory/new",
      SnapshotScope::FILE => "/b/project/file",
      SnapshotScope::MISSING => "/b/project/missing",
      SnapshotScope::BUILD => unreachable!(),
    };
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs.write(changed.into(), b"new input").await.unwrap();
    let (_, modified, _) = context.load_snapshot(&reader).await.unwrap();
    assert!(modified.contains(&moved), "scope={scope:?}");
  }
}

#[tokio::test]
async fn fixed_build_inputs_still_bind_their_build_owner() {
  rspack_tasks::within_compiler_context_for_testing(async {
    let writer = MakeOccasion::new(codec("/a/project", "/a/project"));
    let reader = MakeOccasion::new(codec("/b/project", "/b/project"));
    let logger = CompilationLogger::new("test", Default::default());
    let mut storage = MemoryStorage::default();
    writer.save(
      &mut storage,
      &artifact("/a/project/module.js".into(), "source"),
    );
    let recovered = reader.recovery(&storage, &logger).await.unwrap();
    assert_eq!(recovered.item.module_graph.modules_len(), 1);
    assert!(recovered.dropped.is_empty());
    for path in ["/a/project/input", "/a/outside"] {
      for scope in [
        SnapshotScope::FILE,
        SnapshotScope::CONTEXT,
        SnapshotScope::MISSING,
        SnapshotScope::BUILD,
      ] {
        let identifier = ModuleIdentifier::from("/a/project/module.js");
        let mut item = BuildModuleGraphArtifact::new();
        item
          .module_graph
          .add_module_graph_module(ModuleGraphModule::new(identifier));
        let mut module = RawModule::new(
          "source".into(),
          identifier,
          "module".into(),
          RuntimeGlobals::empty(),
        );
        let dependencies = &mut module.build_info_mut().dependencies;
        let paths = match scope {
          SnapshotScope::FILE => &mut dependencies.file,
          SnapshotScope::CONTEXT => &mut dependencies.context,
          SnapshotScope::MISSING => &mut dependencies.missing,
          SnapshotScope::BUILD => &mut dependencies.build,
        };
        paths.insert(InternedPath::from(path).into_fixed());
        module.freeze_build_info();
        module.freeze_build_meta();
        item
          .module_graph
          .add_module(BoxModule::new(Box::new(module)));
        item.issuer_update_modules.insert(identifier);
        let mut storage = MemoryStorage::default();
        MakeOccasion::new(codec("/a/project", "/a/project")).save(&mut storage, &item);
        let recovered = MakeOccasion::new(codec("/b/project", "/b/project"))
          .recovery(
            &storage,
            &CompilationLogger::new("test", Default::default()),
          )
          .await
          .unwrap();
        assert_eq!(
          recovered.item.module_graph.modules_len(),
          0,
          "{path} {scope:?}"
        );
        assert_eq!(recovered.dropped.len(), 1);
      }
    }
  })
  .await;
}

#[tokio::test]
async fn layout_revalidation_does_not_relax_global_build_dependencies() {
  let fs = Arc::new(MemoryFileSystem::default());
  for root in ["/a/project", "/b/project"] {
    fs.create_dir_all(root.into()).await.unwrap();
    fs.write(format!("{root}/input").as_str().into(), b"same")
      .await
      .unwrap();
  }
  let mut storage = MemoryStorage::default();
  Snapshot::new(
    options(),
    "/a/project".into(),
    fs.clone(),
    codec("/a/project", "/a/project"),
  )
  .add(
    &mut storage,
    SnapshotScope::BUILD,
    [InternedPath::from("/a/project/input").into_fixed()].into_iter(),
  )
  .await;
  let (_, modified, _, unchanged, _, dropped) = Snapshot::new(
    options(),
    "/b/project".into(),
    fs,
    codec("/b/project", "/b/project"),
  )
  .calc_modified_paths(&storage, SnapshotScope::BUILD)
  .await
  .unwrap();
  assert!(modified.contains(&InternedPath::from("/b/project/input")));
  assert!(unchanged.is_empty());
  assert_eq!(dropped.len(), 2);
}

#[tokio::test]
async fn expired_layout_snapshots_still_detect_link_and_package_metadata_changes() {
  use std::{fs, os::unix::fs::symlink};

  use rspack_core::PathMatcher;

  let temp = tempfile::tempdir().unwrap();
  let base = fs::canonicalize(temp.path()).unwrap();
  for root in [base.join("a/project"), base.join("b/project")] {
    fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
    fs::write(root.join("first.js"), "same").unwrap();
    fs::write(root.join("second.js"), "same").unwrap();
    symlink("first.js", root.join("link.js")).unwrap();
    fs::write(
      root.join("node_modules/pkg/package.json"),
      r#"{"name":"pkg","version":"1"}"#,
    )
    .unwrap();
    fs::write(root.join("node_modules/pkg/index.js"), "same").unwrap();
  }
  let writer = base.join("a/project").assert_utf8();
  let reader = base.join("b/project").assert_utf8();
  let mut configuration = options();
  configuration
    .managed_paths
    .push(PathMatcher::String(base.to_string_lossy().into_owned()));
  let mut storage = MemoryStorage::default();
  Snapshot::new(
    configuration.clone(),
    writer.as_str().into(),
    Arc::new(NativeFileSystem::new(false)),
    codec_for_moved(writer.as_str()),
  )
  .add(
    &mut storage,
    SnapshotScope::FILE,
    ["link.js", "node_modules/pkg/index.js"]
      .map(|p| InternedPath::from(writer.join(p).as_std_path()).into_fixed())
      .into_iter(),
  )
  .await;
  let snapshot = Snapshot::new(
    configuration,
    reader.as_str().into(),
    Arc::new(NativeFileSystem::new(false)),
    codec_for_moved(reader.as_str()),
  );
  let (_, modified, _, unchanged, _, _) = snapshot
    .calc_modified_paths(&storage, SnapshotScope::FILE)
    .await
    .unwrap();
  for p in ["link.js", "node_modules/pkg/index.js"] {
    let path = InternedPath::from(reader.join(p).as_std_path());
    assert!(!modified.contains(&path));
    assert!(unchanged.contains(&path));
  }
  fs::remove_file(reader.join("link.js")).unwrap();
  symlink("second.js", reader.join("link.js")).unwrap();
  fs::write(
    reader.join("node_modules/pkg/package.json"),
    r#"{"name":"pkg","version":"2"}"#,
  )
  .unwrap();
  let (_, modified, _, _, _, _) = snapshot
    .calc_modified_paths(&storage, SnapshotScope::FILE)
    .await
    .unwrap();
  assert!(modified.contains(&InternedPath::from(reader.join("link.js").as_std_path())));
  assert!(modified.contains(&InternedPath::from(
    reader.join("node_modules/pkg/index.js").as_std_path()
  )));
}
