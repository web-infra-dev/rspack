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
async fn symlink_contexts_track_unresolvable_links_including_managed_roots() {
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
    Strategy::DirHash { .. }
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
    Strategy::DirHash { .. }
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
async fn missing_file_dependency_stays_valid_until_created() {
  for portable in [false, true] {
    let fs = Arc::new(MemoryFileSystem::default());
    fs.create_dir_all("/project".into()).await.unwrap();
    let codec = if portable {
      codec("/project", "/project")
    } else {
      Arc::new(CacheCodec::new(None))
    };
    let snapshot = Snapshot::new(options(), "/project".into(), fs.clone(), codec);
    let path = InternedPath::from("/project/optional.config.js");
    let mut storage = MemoryStorage::default();
    snapshot
      .add(
        &mut storage,
        SnapshotScope::FILE,
        [path.clone()].into_iter(),
      )
      .await;
    for _ in 0..2 {
      let (_, modified, deleted, unchanged, _, _) = snapshot
        .calc_modified_paths(&storage, SnapshotScope::FILE)
        .await
        .unwrap();
      assert!(modified.is_empty() && deleted.is_empty());
      assert!(unchanged.contains(&path));
    }
    fs.write(path.assert_utf8(), b"module.exports = 1")
      .await
      .unwrap();
    let (_, modified, _, _, _, _) = snapshot
      .calc_modified_paths(&storage, SnapshotScope::FILE)
      .await
      .unwrap();
    assert!(modified.contains(&path));
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

#[tokio::test]
async fn context_with_unresolvable_links_stays_valid_and_detects_retarget() {
  use std::{fs, os::unix::fs::symlink};
  for portable in [false, true] {
    for target in ["missing", ".", "..", "loop"] {
      let temp = tempfile::tempdir().unwrap();
      let root = fs::canonicalize(temp.path()).unwrap().assert_utf8();
      fs::create_dir(root.join("scan")).unwrap();
      fs::write(root.join("scan/a.js"), "a").unwrap();
      symlink(target, root.join("scan/loop")).unwrap();
      let codec = if portable {
        codec_for_moved(root.as_str())
      } else {
        Arc::new(CacheCodec::new(None))
      };
      let snapshot = Snapshot::new(
        options(),
        root.as_str().into(),
        Arc::new(NativeFileSystem::new(false)),
        codec,
      );
      let path = InternedPath::from(root.join("scan").as_std_path());
      let mut storage = MemoryStorage::default();
      snapshot
        .add(
          &mut storage,
          SnapshotScope::CONTEXT,
          [path.clone()].into_iter(),
        )
        .await;
      let (_, modified, deleted, unchanged, _, _) = snapshot
        .calc_modified_paths(&storage, SnapshotScope::CONTEXT)
        .await
        .unwrap();
      assert!(
        modified.is_empty() && deleted.is_empty(),
        "portable={portable}, target={target}"
      );
      assert!(unchanged.contains(&path));
      fs::remove_file(root.join("scan/loop")).unwrap();
      symlink("other-missing", root.join("scan/loop")).unwrap();
      let (_, modified, _, _, _, _) = snapshot
        .calc_modified_paths(&storage, SnapshotScope::CONTEXT)
        .await
        .unwrap();
      assert!(modified.contains(&path));
    }
  }
}

/// Count observations through the same filesystem interface used by the compiler.
#[derive(Debug)]
struct ObservedFileSystem {
  inner: NativeFileSystem,
  watched: rspack_paths::Utf8PathBuf,
  observations: std::sync::atomic::AtomicUsize,
  reads: std::sync::atomic::AtomicUsize,
}

impl ObservedFileSystem {
  fn new(watched: rspack_paths::Utf8PathBuf) -> Self {
    Self {
      inner: NativeFileSystem::new(false),
      watched,
      observations: 0.into(),
      reads: 0.into(),
    }
  }
}

#[async_trait::async_trait]
impl ReadableFileSystem for ObservedFileSystem {
  async fn read(&self, path: &rspack_paths::Utf8Path) -> rspack_fs::Result<Vec<u8>> {
    self
      .reads
      .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    self.inner.read(path).await
  }
  fn read_sync(&self, path: &rspack_paths::Utf8Path) -> rspack_fs::Result<Vec<u8>> {
    self.inner.read_sync(path)
  }
  async fn metadata(
    &self,
    path: &rspack_paths::Utf8Path,
  ) -> rspack_fs::Result<rspack_fs::FileMetadata> {
    self.inner.metadata(path).await
  }
  fn metadata_sync(
    &self,
    path: &rspack_paths::Utf8Path,
  ) -> rspack_fs::Result<rspack_fs::FileMetadata> {
    self.inner.metadata_sync(path)
  }
  async fn symlink_metadata(
    &self,
    path: &rspack_paths::Utf8Path,
  ) -> rspack_fs::Result<rspack_fs::FileMetadata> {
    if path == self.watched {
      self
        .observations
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    self.inner.symlink_metadata(path).await
  }
  async fn read_link(
    &self,
    path: &rspack_paths::Utf8Path,
  ) -> rspack_fs::Result<rspack_paths::Utf8PathBuf> {
    self.inner.read_link(path).await
  }
  async fn canonicalize(
    &self,
    path: &rspack_paths::Utf8Path,
  ) -> rspack_fs::Result<rspack_paths::Utf8PathBuf> {
    self.inner.canonicalize(path).await
  }
  async fn read_dir(&self, path: &rspack_paths::Utf8Path) -> rspack_fs::Result<Vec<String>> {
    ReadableFileSystem::read_dir(&self.inner, path).await
  }
  fn read_dir_sync(&self, path: &rspack_paths::Utf8Path) -> rspack_fs::Result<Vec<String>> {
    self.inner.read_dir_sync(path)
  }
  async fn permissions(
    &self,
    path: &rspack_paths::Utf8Path,
  ) -> rspack_fs::Result<Option<rspack_fs::FilePermissions>> {
    self.inner.permissions(path).await
  }
}

#[tokio::test]
async fn timestamp_context_does_not_read_linked_file_contents() {
  use std::{fs, os::unix::fs::symlink, sync::atomic::Ordering};
  let temp = tempfile::tempdir().unwrap();
  let root = fs::canonicalize(temp.path()).unwrap().assert_utf8();
  fs::create_dir(root.join("scan")).unwrap();
  fs::create_dir_all(root.join("assets/nested")).unwrap();
  fs::write(root.join("assets/nested/a.bin"), "asset").unwrap();
  symlink("../assets", root.join("scan/assets")).unwrap();
  let fs = Arc::new(ObservedFileSystem::new(root.join("scan")));
  let snapshot = Snapshot::new(
    options(),
    root.as_str().into(),
    fs.clone(),
    Arc::new(CacheCodec::new(None)),
  );
  let path = InternedPath::from(root.join("scan").as_std_path());
  let mut storage = MemoryStorage::default();
  snapshot
    .add(
      &mut storage,
      SnapshotScope::CONTEXT,
      [path.clone()].into_iter(),
    )
    .await;
  fs.reads.store(0, Ordering::Relaxed);
  let (_, modified, deleted, unchanged, _, _) = snapshot
    .calc_modified_paths(&storage, SnapshotScope::CONTEXT)
    .await
    .unwrap();
  assert!(modified.is_empty() && deleted.is_empty());
  assert!(unchanged.contains(&path));
  assert_eq!(
    fs.reads.load(Ordering::Relaxed),
    0,
    "timestamp validation must not read asset content"
  );
  // A context dependency must still notice changes below the linked directory.
  std::fs::write(root.join("assets/nested/added.bin"), "new asset").unwrap();
  let (_, modified, _, _, _, _) = snapshot
    .calc_modified_paths(&storage, SnapshotScope::CONTEXT)
    .await
    .unwrap();
  assert!(modified.contains(&path));
}

#[tokio::test]
async fn context_validation_keeps_its_single_observation() {
  use std::{fs, sync::atomic::Ordering};
  let temp = tempfile::tempdir().unwrap();
  let root = fs::canonicalize(temp.path()).unwrap().assert_utf8();
  fs::create_dir(root.join("scan")).unwrap();
  fs::write(root.join("scan/a.js"), "a").unwrap();
  let fs = Arc::new(ObservedFileSystem::new(root.join("scan")));
  let codec = Arc::new(CacheCodec::new(None));
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
  fs.observations.store(0, Ordering::Relaxed);
  let (_, modified, deleted, unchanged, observed, _) = snapshot
    .calc_modified_paths(&storage, SnapshotScope::CONTEXT)
    .await
    .unwrap();
  assert!(modified.is_empty() && deleted.is_empty());
  assert!(unchanged.contains(&path));
  assert_eq!(
    fs.observations.load(Ordering::Relaxed),
    1,
    "validation must observe a context only once"
  );
  // Saving validation's observation must not hide an input changed after it was read.
  std::fs::write(root.join("scan/b.js"), "b").unwrap();
  snapshot.refresh(&mut storage, SnapshotScope::CONTEXT, observed.into_iter());
  let (_, modified, _, _, _, _) = snapshot
    .calc_modified_paths(&storage, SnapshotScope::CONTEXT)
    .await
    .unwrap();
  assert!(modified.contains(&path));
}

#[test]
fn equal_link_snapshots_compare_equal() {
  assert_eq!(
    Strategy::LinkHash {
      hash: 42,
      deep: false
    },
    Strategy::LinkHash {
      hash: 42,
      deep: false
    }
  );
  assert_ne!(
    Strategy::LinkHash {
      hash: 42,
      deep: false
    },
    Strategy::LinkHash {
      hash: 42,
      deep: true
    }
  );
  assert_ne!(
    Strategy::LinkHash {
      hash: 42,
      deep: false
    },
    Strategy::LinkHash {
      hash: 43,
      deep: false
    }
  );
}

#[tokio::test]
async fn inaccessible_file_dependency_does_not_become_missing() {
  use std::{fs, os::unix::fs::PermissionsExt};
  let temp = tempfile::tempdir().unwrap();
  let root = fs::canonicalize(temp.path()).unwrap().assert_utf8();
  let directory = root.join("private");
  fs::create_dir(&directory).unwrap();
  fs::write(directory.join("config.js"), "config").unwrap();
  let codec = Arc::new(CacheCodec::new(None));
  let snapshot = Snapshot::new(
    options(),
    root.as_str().into(),
    Arc::new(NativeFileSystem::new(false)),
    codec.clone(),
  );
  let mut storage = MemoryStorage::default();
  // ENOTDIR also means the dependency does not exist, rather than an inspection failure.
  let missing = InternedPath::from(directory.join("config.js/child").as_std_path());
  snapshot
    .add(
      &mut storage,
      SnapshotScope::FILE,
      [missing.clone()].into_iter(),
    )
    .await;
  let (_, modified, deleted, unchanged, _, _) = snapshot
    .calc_modified_paths(&storage, SnapshotScope::FILE)
    .await
    .unwrap();
  assert!(modified.is_empty() && deleted.is_empty());
  assert!(unchanged.contains(&missing));
  storage.reset(SnapshotScope::FILE.name());
  fs::set_permissions(&directory, fs::Permissions::from_mode(0o0)).unwrap();
  snapshot
    .add(
      &mut storage,
      SnapshotScope::FILE,
      [InternedPath::from(
        directory.join("config.js").as_std_path(),
      )]
      .into_iter(),
    )
    .await;
  fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
  let entries = storage.load(SnapshotScope::FILE.name()).await.unwrap();
  assert!(matches!(
    codec
      .decode::<(Option<u64>, Strategy)>(&entries[0].1)
      .unwrap()
      .1,
    Strategy::Failed
  ));
}
