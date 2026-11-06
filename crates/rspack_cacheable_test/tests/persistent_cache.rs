#![cfg(unix)]
#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use rspack_core::{
  BoxModule, BuildModuleGraphArtifact, CompilationLogger, Module, ModuleGraphModule,
  ModuleIdentifier, PortableRoot, RawModule, RuntimeGlobals, SnapshotOptions,
  SnapshotStrategyOptions,
  cache::CacheCodec,
  legacy_cache::persistent::{
    context::CacheContext,
    occasion::{MakeOccasion, Occasion},
    snapshot::{Snapshot, SnapshotScope, Strategy},
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
    codec.decode::<Strategy>(&entries[0].1).unwrap(),
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
    decoder.decode::<Strategy>(&entries[0].1).unwrap(),
    Strategy::Failed
  ));
  fs::remove_file(root.join("scan/loop")).unwrap();
  symlink("missing", root.join("scan/broken")).unwrap();
  snapshot
    .add(&mut storage, SnapshotScope::CONTEXT, [path].into_iter())
    .await;
  let entries = storage.load(SnapshotScope::CONTEXT.name()).await.unwrap();
  assert!(matches!(
    decoder.decode::<Strategy>(&entries[0].1).unwrap(),
    Strategy::Failed
  ));
  // The snapshot root itself is a link under managedPaths, so package shortcuts must not hide it.
  let path = InternedPath::from(root.join("scan/broken").as_std_path());
  snapshot
    .add(&mut storage, SnapshotScope::FILE, [path].into_iter())
    .await;
  let entries = storage.load(SnapshotScope::FILE.name()).await.unwrap();
  assert!(matches!(
    decoder.decode::<Strategy>(&entries[0].1).unwrap(),
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
