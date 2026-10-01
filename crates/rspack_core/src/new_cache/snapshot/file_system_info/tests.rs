//! Ported from webpack/test/FileSystemInfo.unittest.js (MIT licensed):
//! https://github.com/webpack/webpack/blob/fc90789fdf089a5de71671f8d10155fcd32dc027/test/FileSystemInfo.unittest.js
//!
//! Keep webpack's assertions as the compatibility target. Tests for missing Rust
//! interfaces use explicit `todo!`s; behavioral mismatches remain failing tests.
//! Remove each TODO's `should_panic` attribute when implementing that test.
//! Run the baseline with `cargo test -p rspack_core file_system_info::tests`.
//! The fixtures use logical time and an in-memory filesystem, including symlinks.

mod fs;
mod hash;
mod timestamp;
mod tsh;

use std::{
  io::ErrorKind,
  sync::{Arc, Mutex},
  time::Duration,
};

use fs::{Operation, TestFileSystem};
use rspack_cacheable::{from_bytes, to_bytes};
use rspack_hash::HashFunction;
use rspack_paths::{InternedPath, InternedPathSet, Utf8Path};
use rspack_regex::RspackRegex;

use super::{FileHash, FileSystemInfo, PathClassification, Snapshot, SnapshotValidationResult};
use crate::{
  InfrastructureLogEvent, InfrastructureLogSink, InfrastructureLogger, LogType, PathMatcher,
  SnapshotOptions, SnapshotStrategyOptions,
};

const START_TIME: u64 = 1_000_000_000;
const FILES: &[&str] = &[
  "/path/file.txt",
  "/path/nested/deep/file.txt",
  "/path/nested/deep/ignored.txt",
  "/path/context+files/file.txt",
  "/path/context+files/sub/file.txt",
  "/path/context+files/sub/ignored.txt",
  "/path/node_modules/package/file.txt",
  "/path/cache/package-1234/file.txt",
  "/path/circular/circular/file2.txt",
  "/path/nested/deep/symlink/file.txt",
  "/path/context+files/sub/symlink/file.txt",
  "/path/context/sub/symlink/file.txt",
  "/path/missing.txt",
  "/path/node_modules/@foo/package1/index.js",
  "/path/node_modules/@foo/package2/index.js",
  "/path/node_modules/bar-package3/index.js",
];
const DIRECTORIES: &[&str] = &[
  "/path/context+files",
  "/path/context",
  "/path/missing",
  "/path/node_modules/package",
  "/path/node_modules/missing",
  "/path/node_modules/@foo",
  "/path/node_modules/@foo/package1",
  "/path/node_modules/@foo/package2",
  "/path/node_modules/bar-package3",
  "/path/cache/package-1234",
  "/path/cache/package-missing",
];
const MISSING: &[&str] = &[
  "/path/package.json",
  "/path/file2.txt",
  "/path/context+files/file2.txt",
  "/path/node_modules/package.txt",
  "/path/node_modules/package/missing.txt",
  "/path/cache/package-2345",
  "/path/cache/package-1234/missing.txt",
  "/path/ignored.txt",
];
const UNMANAGED_PATHS: &[&str] = &[
  "/path/node_modules/@foo/package1",
  "/path/node_modules/@foo/package2",
  "/path/node_modules/bar-package3",
];

#[derive(Default)]
struct TestLog(Mutex<Vec<String>>);

impl InfrastructureLogSink for TestLog {
  fn emit(&self, event: InfrastructureLogEvent) {
    match event.log_type {
      LogType::Error { message, .. } => panic!("{message}"),
      LogType::Warn { message, .. } => self.0.lock().expect("log lock").push(message),
      _ => {}
    }
  }
}

// webpack: createFs (L80). Preserve the paths, contents and links from memfs.
fn create_fs() -> Arc<TestFileSystem> {
  let fs = Arc::new(TestFileSystem::new());
  for dir in [
    "/path/context+files/sub",
    "/path/context/sub",
    "/path/nested/deep",
    "/path/node_modules/package",
    "/path/node_modules/@foo/package1",
    "/path/node_modules/@foo/package2",
    "/path/node_modules/bar-package3",
    "/path/cache/package-1234",
    "/path/folder/context",
    "/path/folder/context+files",
    "/path/folder/nested",
  ] {
    fs.mkdir(dir);
  }
  for (path, content) in [
    ("/path/file.txt", "Hello World"),
    ("/path/file2.txt", "Hello World2"),
    ("/path/nested/deep/file.txt", "Hello World"),
    ("/path/nested/deep/ignored.txt", "Ignored"),
    ("/path/context+files/file.txt", "Hello World"),
    ("/path/context+files/file2.txt", "Hello World2"),
    ("/path/context+files/sub/file.txt", "Hello World"),
    ("/path/context+files/sub/file2.txt", "Hello World2"),
    ("/path/context+files/sub/file3.txt", "Hello World3"),
    ("/path/context+files/sub/ignored.txt", "Ignored"),
    ("/path/context/file.txt", "Hello World"),
    ("/path/context/file2.txt", "Hello World2"),
    ("/path/context/sub/file.txt", "Hello World"),
    ("/path/context/sub/file2.txt", "Hello World2"),
    ("/path/context/sub/file3.txt", "Hello World3"),
    ("/path/context/sub/ignored.txt", "Ignored"),
    ("/path/node_modules/package/file.txt", "Hello World"),
    ("/path/node_modules/package/ignored.txt", "Ignored"),
    ("/path/cache/package-1234/file.txt", "Hello World"),
    ("/path/cache/package-1234/ignored.txt", "Ignored"),
    ("/path/folder/context/file.txt", "Hello World"),
    ("/path/folder/context+files/file.txt", "Hello World"),
    ("/path/folder/nested/file.txt", "Hello World"),
    ("/path/node_modules/@foo/package1/index.js", "Hello World"),
    ("/path/node_modules/@foo/package2/index.js", "Hello World"),
    ("/path/node_modules/bar-package3/index.js", "Hello World"),
  ] {
    fs.write(path, content);
  }
  for path in [
    "/path/node_modules/package/package.json",
    "/path/cache/package-1234/package.json",
  ] {
    fs.write(path, r#"{"name":"package","version":"1.0.0"}"#);
  }
  for (target, link) in [
    ("/path", "/path/circular"),
    ("/path/folder/context", "/path/context/sub/symlink"),
    (
      "/path/folder/context+files",
      "/path/context+files/sub/symlink",
    ),
    ("/path/folder/nested", "/path/nested/deep/symlink"),
  ] {
    fs.symlink(target, link);
  }
  fs
}

fn paths(values: &[&str]) -> InternedPathSet {
  values
    .iter()
    .map(|path| InternedPath::from(*path))
    .collect()
}

fn options() -> SnapshotOptions {
  SnapshotOptions {
    unmanaged_paths: UNMANAGED_PATHS
      .iter()
      .map(|path| PathMatcher::String((*path).into()))
      .collect(),
    managed_paths: vec![PathMatcher::String("/path/node_modules".into())],
    immutable_paths: vec![PathMatcher::String("/path/cache".into())],
    module: SnapshotStrategyOptions::timestamp(),
    context_module: SnapshotStrategyOptions::timestamp(),
    resolve: SnapshotStrategyOptions::timestamp(),
    build_dependencies: SnapshotStrategyOptions::hash(),
    resolve_build_dependencies: SnapshotStrategyOptions::hash(),
  }
}

fn fs_info_with_options(
  fs: Arc<TestFileSystem>,
  options: SnapshotOptions,
) -> (FileSystemInfo, Arc<TestLog>) {
  let log = Arc::new(TestLog::default());
  let logger = InfrastructureLogger::new("FileSystemInfo.unittest", log.clone());
  (
    FileSystemInfo::new(fs, logger, options, HashFunction::SHA256),
    log,
  )
}

// webpack: createFsInfo (L151). Its addFileTimestamps(..., "ignore") setup has
// no Rust equivalent yet. Tests that depend on that setup are explicit TODOs;
// the other tests leave these files unchanged or exercise hash-only tracking.
fn create_fs_info(fs: Arc<TestFileSystem>) -> FileSystemInfo {
  fs_info_with_options(fs, options()).0
}

async fn snapshot_paths(
  fs_info: &FileSystemInfo,
  files: &[&str],
  directories: &[&str],
  missing: &[&str],
  strategy: SnapshotStrategyOptions,
) -> Snapshot {
  fs_info
    .create_snapshot(
      Some(START_TIME),
      &paths(files),
      &paths(directories),
      &paths(missing),
      strategy,
    )
    .await
    .expect("webpack should create a snapshot")
}

async fn snapshot(fs_info: &FileSystemInfo, strategy: SnapshotStrategyOptions) -> Snapshot {
  snapshot_paths(fs_info, FILES, DIRECTORIES, MISSING, strategy).await
}

// webpack: clone uses buffersSerializer. Exercise the actual Rust cache codec,
// not Snapshot::clone, so indirect validation also covers serialization.
fn round_trip(snapshot: &Snapshot) -> Snapshot {
  let bytes = to_bytes(snapshot, &()).expect("serialize snapshot");
  from_bytes(&bytes, &()).expect("deserialize snapshot")
}

// webpack: expectSnapshotState (L234), including direct and serialized reuse.
async fn expect_snapshot_state(fs: Arc<TestFileSystem>, snapshot: &Snapshot, expected: bool) {
  let fs_info = create_fs_info(fs);
  for attempt in ["initial", "directly cached"] {
    let result = fs_info
      .check_snapshot_valid(snapshot)
      .await
      .expect("snapshot validation");
    assert_eq!(
      matches!(result, SnapshotValidationResult::Valid),
      expected,
      "{attempt}: {result:?}"
    );
  }
  let restored = round_trip(snapshot);
  let result = fs_info
    .check_snapshot_valid(&restored)
    .await
    .expect("restored snapshot validation");
  assert_eq!(
    matches!(result, SnapshotValidationResult::Valid),
    expected,
    "serialized: {result:?}"
  );
}

// webpack: updateFile (L295).
fn update_file(fs: &TestFileSystem, filename: &str) {
  use rspack_fs::ReadableFileSystem;
  let content = fs
    .read_to_string_sync(Utf8Path::new(filename))
    .expect("fixture file");
  if filename.ends_with(".json") {
    let mut data: serde_json::Value = serde_json::from_str(&content).expect("fixture JSON");
    data["version"] = format!("{}.1", data["version"].as_str().expect("version")).into();
    fs.write(filename, serde_json::to_vec(&data).expect("updated JSON"));
  } else {
    fs.write(filename, format!("{content}!"));
  }
}

async fn check_change(strategy: SnapshotStrategyOptions, path: &str, create: bool, expected: bool) {
  let fs = create_fs();
  let fs_info = create_fs_info(fs.clone());
  let initial = snapshot(&fs_info, strategy).await;
  let cached = snapshot(&fs_info, strategy).await;
  if create {
    fs.write(path, "New file");
  } else {
    update_file(&fs, path);
  }
  for snapshot in [&initial, &cached] {
    expect_snapshot_state(fs.clone(), snapshot, expected).await;
  }
}

async fn check_empty_snapshot(strategy: SnapshotStrategyOptions) {
  let fs_info = create_fs_info(create_fs());
  let snapshot = snapshot_paths(&fs_info, &[], &[], &[], strategy).await;
  expect_snapshot_state(create_fs(), &snapshot, true).await;
}

async fn check_unchanged_snapshot(strategy: SnapshotStrategyOptions) {
  let fs = create_fs();
  let fs_info = create_fs_info(fs.clone());
  let initial = snapshot(&fs_info, strategy).await;
  let cached = snapshot(&fs_info, strategy).await;
  for snapshot in [&initial, &cached] {
    expect_snapshot_state(fs.clone(), snapshot, true).await;
  }
}

async fn check_changed_timestamps(strategy: SnapshotStrategyOptions) {
  let fs = create_fs();
  let fs_info = create_fs_info(fs.clone());
  let initial = snapshot(&fs_info, strategy).await;
  let cached = snapshot(&fs_info, strategy).await;
  // Unlike wall-clock memfs, explicitly guarantee different mtimes.
  fs.advance_timestamps(1_000_000);
  for snapshot in [&initial, &cached] {
    expect_snapshot_state(fs.clone(), snapshot, true).await;
  }
}

// webpack: "stable iterables identity" (L452).
mod stable_iterables_identity {
  #[test]
  #[should_panic(
    expected = "webpack L471: Snapshot has no getFileIterable equivalent or cached iterable identity; verify repeated calls return the same iterable"
  )]
  fn should_return_same_iterable_for_get_file_iterable() {
    todo!(
      "{}",
      "webpack L471: Snapshot has no getFileIterable equivalent or cached iterable identity; verify repeated calls return the same iterable"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L481: Snapshot has no getContextIterable equivalent or cached iterable identity; verify repeated calls return the same iterable"
  )]
  fn should_return_same_iterable_for_get_context_iterable() {
    todo!(
      "{}",
      "webpack L481: Snapshot has no getContextIterable equivalent or cached iterable identity; verify repeated calls return the same iterable"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L491: Snapshot has no getMissingIterable equivalent or cached iterable identity; cover missing dependencies when the API is added"
  )]
  fn should_return_same_iterable_for_get_missing_iterable() {
    todo!(
      "{}",
      "webpack L491: Snapshot has no getMissingIterable equivalent or cached iterable identity; cover missing dependencies when the API is added"
    );
  }
}

// webpack: "path classification cache" (L502).
mod path_classification_cache {
  use super::*;

  #[tokio::test]
  #[should_panic(expected = "not yet implemented: webpack L574-L586:")]
  async fn should_memoize_reg_exp_path_classification_across_snapshots() {
    let fs = create_fs();
    let mut options = options();
    options.unmanaged_paths = UNMANAGED_PATHS
      .iter()
      .map(|path| {
        PathMatcher::Regexp(RspackRegex::new(&format!("^{path}/")).expect("unmanaged regexp"))
      })
      .collect();
    options.managed_paths = vec![PathMatcher::Regexp(
      RspackRegex::new(r"^(/path/node_modules/)").expect("managed regexp"),
    )];
    options.immutable_paths = vec![PathMatcher::Regexp(
      RspackRegex::new(r"^/path/cache/").expect("immutable regexp"),
    )];
    let (fs_info, _) = fs_info_with_options(fs, options);
    let first = snapshot(&fs_info, SnapshotStrategyOptions::timestamp()).await;
    let cache = &fs_info.inner.path_classification_cache;
    for path in [
      "/path/file.txt",
      "/path/node_modules/@foo/package1/index.js",
    ] {
      assert!(matches!(
        *cache.get(&InternedPath::from(path)).expect("classified"),
        PathClassification::Unmanaged
      ));
    }
    assert!(matches!(
      *cache
        .get(&InternedPath::from("/path/cache/package-1234/file.txt"))
        .expect("classified"),
      PathClassification::Immutable
    ));
    assert!(matches!(
      &*cache.get(&InternedPath::from("/path/node_modules/package/file.txt")).expect("classified"),
      PathClassification::Managed(item) if *item == InternedPath::from("/path/node_modules/package")
    ));
    let managed = first.managed_files.as_ref().expect("managed files");
    assert!(managed.contains(&InternedPath::from("/path/node_modules/package/file.txt")));
    assert!(managed.contains(&InternedPath::from("/path/cache/package-1234/file.txt")));
    let cache_size = cache.len();
    let second = snapshot(&fs_info, SnapshotStrategyOptions::timestamp()).await;
    assert_eq!(cache.len(), cache_size);
    assert_eq!(first.file_timestamps, second.file_timestamps);
    assert_eq!(first.managed_files, second.managed_files);
    assert_eq!(first.managed_contexts, second.managed_contexts);
    assert_eq!(first.managed_missing, second.managed_missing);
    todo!(
      "webpack L574-L586: remaining assertions need Snapshot's file/context/missing iterables and FileSystemInfo.clear() to verify capture completeness and classification-cache reset"
    );
  }
}

// webpack: "per-path cache reuse" (L594).
mod per_path_cache_reuse {
  use super::*;

  async fn check(strategy: SnapshotStrategyOptions) {
    let fs = create_fs();
    let fs_info = create_fs_info(fs.clone());
    let two_files = &["/path/file.txt", "/path/nested/deep/file.txt"];
    snapshot_paths(&fs_info, two_files, &[], &[], strategy).await;
    let before = fs.calls();
    let second = snapshot_paths(&fs_info, two_files, &[], &[], strategy).await;
    // Observe cache reuse through the filesystem boundary. No private queue or
    // optimizer implementation is required to prove that no extra I/O occurred.
    assert_eq!(
      fs.calls(),
      before,
      "the second snapshot should reuse per-path info"
    );
    let captured: InternedPathSet = if strategy.hash && strategy.timestamp {
      second
        .file_timestamp_hashes
        .as_ref()
        .expect("tsh entries")
        .keys()
        .cloned()
        .collect()
    } else if strategy.hash {
      second
        .file_hashes
        .as_ref()
        .expect("hash entries")
        .keys()
        .cloned()
        .collect()
    } else {
      second
        .file_timestamps
        .as_ref()
        .expect("timestamp entries")
        .keys()
        .cloned()
        .collect()
    };
    assert_eq!(captured, paths(two_files));
    expect_snapshot_state(fs, &second, true).await;
  }

  #[tokio::test]
  async fn should_reuse_cached_file_info_in_timestamp_mode_below_the_sharing_threshold() {
    check(SnapshotStrategyOptions::timestamp()).await;
  }
  #[tokio::test]
  async fn should_reuse_cached_file_info_in_hash_mode_below_the_sharing_threshold() {
    check(SnapshotStrategyOptions::hash()).await;
  }
  #[tokio::test]
  async fn should_reuse_cached_file_info_in_tsh_mode_below_the_sharing_threshold() {
    check(SnapshotStrategyOptions::hash_and_timestamp()).await;
  }
}

// webpack: "symlinks" (L635).
mod symlinks {
  use super::*;

  #[tokio::test]
  #[should_panic] // We don't support read_link retries yet
  async fn should_work_with_symlinks_with_errors() {
    let fs = create_fs();
    let link = "/path/context/sub/symlink-error";
    fs.symlink("/path/folder/context", link);
    // webpack L650: fail the first two readlink calls before the link becomes readable.
    fs.fail(Operation::ReadLink, link, ErrorKind::Other, 2);
    let fs_info = create_fs_info(fs.clone());
    let initial = snapshot(&fs_info, SnapshotStrategyOptions::timestamp()).await;
    let cached = snapshot(&fs_info, SnapshotStrategyOptions::timestamp()).await;
    for snapshot in [&initial, &cached] {
      expect_snapshot_state(fs.clone(), snapshot, true).await;
    }
  }

  #[test]
  #[should_panic(
    expected = "webpack L674: permanent readlink errors must abort creation with a null snapshot; Rust has no successful no-snapshot return (create_snapshot returns Result<Snapshot>)"
  )]
  fn should_work_with_symlinks_with_errors_1() {
    todo!(
      "{}",
      "webpack L674: permanent readlink errors must abort creation with a null snapshot; Rust has no successful no-snapshot return (create_snapshot returns Result<Snapshot>)"
    );
  }

  #[tokio::test]
  async fn should_terminate_on_a_cyclic_symlink_graph() {
    let fs = create_fs();
    fs.write("/path/cycle/a/file.txt", "Hello A");
    fs.write("/path/cycle/b/file.txt", "Hello B");
    fs.symlink("../b", "/path/cycle/a/link");
    fs.symlink("../a", "/path/cycle/b/link");
    let fs_info = create_fs_info(fs);
    let path = InternedPath::from("/path/cycle/a");
    let hash = tokio::time::timeout(Duration::from_secs(5), fs_info.context_hash(&path))
      .await
      .expect("cyclic symlink graph must terminate")
      .expect("context hash");
    assert!(hash.is_some());
  }

  #[tokio::test]
  async fn should_invalidate_when_a_file_behind_an_absolute_symlink_target_changes() {
    let fs = create_fs();
    let fs_info = create_fs_info(fs.clone());
    let snapshot = snapshot_paths(
      &fs_info,
      &[],
      &["/path/context/sub"],
      &[],
      SnapshotStrategyOptions::hash(),
    )
    .await;
    fs.write("/path/folder/context/file.txt", "Changed");
    expect_snapshot_state(fs, &snapshot, false).await;
  }

  #[tokio::test]
  async fn should_not_crash_on_a_symlink_whose_target_directory_is_missing() {
    let fs = create_fs();
    fs.symlink("/path/missing", "/path/context/sub/dangling");
    let fs_info = create_fs_info(fs);
    let missing = InternedPath::from("/path/missing");
    assert!(
      fs_info
        .context_timestamp(&missing)
        .await
        .expect("missing timestamp")
        .is_none()
    );
    assert!(
      fs_info
        .context_hash(&missing)
        .await
        .expect("missing hash")
        .is_none()
    );
    let tsh = fs_info
      .context_timestamp_and_hash(&InternedPath::from("/path/context/sub"))
      .await
      .expect("dangling symlink must not abort context hashing")
      .expect("context exists");
    assert!(!tsh.hash.encoded().is_empty());
  }

  #[tokio::test]
  async fn should_invalidate_a_snapshot_when_a_missing_directory_is_created() {
    let fs = create_fs();
    let fs_info = create_fs_info(fs.clone());
    let missing = InternedPath::from("/path/missing");
    assert!(
      fs_info
        .context_timestamp(&missing)
        .await
        .expect("timestamp")
        .is_none()
    );
    assert!(
      fs_info
        .context_hash(&missing)
        .await
        .expect("hash")
        .is_none()
    );
    let snapshot = snapshot_paths(
      &fs_info,
      &[],
      &["/path/missing"],
      &[],
      SnapshotStrategyOptions::hash_and_timestamp(),
    )
    .await;
    fs.write("/path/missing/file.txt", "Hello World");
    expect_snapshot_state(fs, &snapshot, false).await;
  }
}

// webpack: "unsupported directory entries" (L792).
mod unsupported_directory_entries {
  use super::*;

  #[tokio::test]
  async fn should_hash_a_context_containing_an_unsupported_entry() {
    let fs = create_fs();
    fs.write("/path/special/file.txt", "Hello World");
    // The upstream fixture likewise overrides the file's lstat type while
    // leaving the empty file readable, rather than opening a real FIFO/socket.
    fs.special("/path/special/socket");
    let path = InternedPath::from("/path/special");
    let hash = create_fs_info(fs.clone())
      .context_hash(&path)
      .await
      .expect("context hash")
      .expect("context exists");
    let tsh = create_fs_info(fs)
      .context_timestamp_and_hash(&path)
      .await
      .expect("context tsh")
      .expect("context exists");
    assert_eq!(hash, tsh.hash);
  }
}

// webpack: "existence-only watchpack entries" (L834).
mod existence_only_watchpack_entries {
  use super::*;

  #[test]
  #[should_panic(
    expected = "webpack L858: addContextTimestamps must accept {} and re-read directory timestamps; context cache currently requires a complete timestamp_hash"
  )]
  fn keeps_snapshot_valid_when_watchpack_reports_existence_only_context_dirs() {
    todo!(
      "{}",
      "webpack L858: addContextTimestamps must accept {} and re-read directory timestamps; context cache currently requires a complete timestamp_hash"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L875: addFileTimestamps must accept {} and re-stat files; the Rust watcher-entry injection API/state is missing"
  )]
  fn keeps_snapshot_valid_when_watchpack_reports_existence_only_files() {
    todo!(
      "{}",
      "webpack L875: addFileTimestamps must accept {} and re-stat files; the Rust watcher-entry injection API/state is missing"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L892: inject { safeTime: 1 } for /path/context+files and complete its missing timestampHash from disk; partial context timestamp state is missing"
  )]
  fn keeps_snapshot_valid_when_watchpack_reports_context_safe_time_without_hash() {
    todo!(
      "{}",
      "webpack L892: inject { safeTime: 1 } for /path/context+files and complete its missing timestampHash from disk; partial context timestamp state is missing"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L913: inject { safeTime: startTime + 5000 } and reject the snapshot; addContextTimestamps and partial context entries are missing"
  )]
  fn invalidates_when_watchpack_safe_time_is_newer_than_snapshot_start() {
    todo!(
      "{}",
      "webpack L913: inject { safeTime: startTime + 5000 } and reject the snapshot; addContextTimestamps and partial context entries are missing"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L944: seed context cache with {} then require a timestampHash; cannot represent an existence-only context cache entry yet"
  )]
  fn get_context_timestamp_falls_back_to_disk_for_existence_only_cache() {
    todo!(
      "{}",
      "webpack L944: seed context cache with {} then require a timestampHash; cannot represent an existence-only context cache entry yet"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L958: seed file cache with {} then require a timestamp; cannot distinguish watcher existence-only info from a directory timestamp yet"
  )]
  fn get_file_timestamp_falls_back_to_disk_for_existence_only_cache() {
    todo!(
      "{}",
      "webpack L958: seed file cache with {} then require a timestamp; cannot distinguish watcher existence-only info from a directory timestamp yet"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L972: addFileTimestamps({}) for absent /path/package.json must re-stat instead of claiming existence; watcher injection/state is missing"
  )]
  fn keeps_snapshot_valid_for_an_existence_only_missing_dependency() {
    todo!(
      "{}",
      "webpack L972: addFileTimestamps({}) for absent /path/package.json must re-stat instead of claiming existence; watcher injection/state is missing"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1033: inject ignore for /path/context+files and return ignore from the getter; context timestamp type has no ignore variant"
  )]
  fn get_context_timestamp_returns_ignore_for_an_ignored_context_dir() {
    todo!(
      "{}",
      "webpack L1033: inject ignore for /path/context+files and return ignore from the getter; context timestamp type has no ignore variant"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1045: inject { safeTime: 1 } before snapshot creation and assert stored timestampHash; partial context cache entries are not representable"
  )]
  fn snapshot_creation_rereads_context_dirs_without_timestamp_hash() {
    todo!(
      "{}",
      "webpack L1045: inject { safeTime: 1 } before snapshot creation and assert stored timestampHash; partial context cache entries are not representable"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1082 (#21378): inject { safeTime, timestamp } for /root/real and /root/real/sub behind ../real; addContextTimestamps and unresolved context entries are missing"
  )]
  fn check_snapshot_valid_resolves_symlink_targets_with_watchpack_timestamps() {
    todo!(
      "{}",
      "webpack L1082 (#21378): inject { safeTime, timestamp } for /root/real and /root/real/sub behind ../real; addContextTimestamps and unresolved context entries are missing"
    );
  }

  // This upstream case does not inject watcher entries and is runnable now.
  #[tokio::test]
  async fn invalidates_when_a_previously_missing_dependency_now_exists() {
    let fs = create_fs();
    let initial = snapshot(
      &create_fs_info(fs.clone()),
      SnapshotStrategyOptions::timestamp(),
    )
    .await;
    fs.write("/path/package.json", "{}");
    expect_snapshot_state(fs, &initial, false).await;
  }
}

// webpack: "ignored context entries" (L1142).
mod ignored_context_entries {
  #[test]
  #[should_panic(
    expected = "webpack L1152: change /path/context+files/sub/file3.txt, prove invalid, then inject context ignore and require valid; timestamp cache has no ignore state"
  )]
  fn keeps_snapshot_valid_when_a_tracked_context_dir_becomes_ignored_timestamp() {
    todo!(
      "{}",
      "webpack L1152: change /path/context+files/sub/file3.txt, prove invalid, then inject context ignore and require valid; timestamp cache has no ignore state"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1152: the timestamp+hash validator must bypass an ignored /path/context+files directory after its contents change; context ignore state is missing"
  )]
  fn keeps_snapshot_valid_when_a_tracked_context_dir_becomes_ignored_tsh() {
    todo!(
      "{}",
      "webpack L1152: the timestamp+hash validator must bypass an ignored /path/context+files directory after its contents change; context ignore state is missing"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1192: addContextTimestamps(ignore) before creation and assert no /path/context+files timestamp entry; injection API and ignore state are missing"
  )]
  fn omits_ignored_context_dirs_from_a_fresh_snapshot() {
    todo!(
      "{}",
      "webpack L1192: addContextTimestamps(ignore) before creation and assert no /path/context+files timestamp entry; injection API and ignore state are missing"
    );
  }
}

// webpack: "cache maintenance" (L1214).
mod cache_maintenance {
  #[test]
  #[should_panic(
    expected = "webpack L1249: build two overlapping snapshots and assert logStatistics reports new snapshots; Rust has no statistics counters or log_statistics method"
  )]
  fn log_statistics_logs_cache_and_optimization_stats() {
    todo!(
      "{}",
      "webpack L1249: build two overlapping snapshots and assert logStatistics reports new snapshots; Rust has no statistics counters or log_statistics method"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1263: populate caches, clear, assert timestamp/managed caches and snapshot statistics are empty; FileSystemInfo.clear and statistics are missing"
  )]
  fn clear_empties_caches_and_resets_stats() {
    todo!(
      "{}",
      "webpack L1263: populate caches, clear, assert timestamp/managed caches and snapshot statistics are empty; FileSystemInfo.clear and statistics are missing"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1280: require a memoized legacy timestamp map with ignored=null and missing omitted; legacy getter and ignore cache state are missing"
  )]
  fn get_deprecated_file_timestamps_reflects_cache_and_is_memoized() {
    todo!(
      "{}",
      "webpack L1280: require a memoized legacy timestamp map with ignored=null and missing omitted; legacy getter and ignore cache state are missing"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1301: require repeated legacy context timestamp getters to return the same populated map; the legacy view API is missing"
  )]
  fn get_deprecated_context_timestamps_reflects_cache_and_is_memoized() {
    todo!(
      "{}",
      "webpack L1301: require repeated legacy context timestamp getters to return the same populated map; the legacy view API is missing"
    );
  }
}

// webpack: "mergeSnapshots" (L1314).
mod merge_snapshots {
  use super::*;

  #[tokio::test]
  #[should_panic(expected = "not yet implemented: webpack L1348:")]
  async fn merges_two_cached_snapshots_into_a_valid_cached_snapshot() {
    let fs = create_fs();
    let fs_info = create_fs_info(fs);
    let first = snapshot(&fs_info, SnapshotStrategyOptions::hash_and_timestamp()).await;
    let second = fs_info
      .create_snapshot(
        Some(START_TIME + 10_000),
        &paths(FILES),
        &paths(DIRECTORIES),
        &paths(MISSING),
        SnapshotStrategyOptions::hash_and_timestamp(),
      )
      .await
      .expect("second snapshot");
    let merged = fs_info.merge_snapshots(first, second);
    assert_eq!(merged.start_time, Some(START_TIME));
    assert!(matches!(
      fs_info
        .check_snapshot_valid(&merged)
        .await
        .expect("merged snapshot"),
      SnapshotValidationResult::Valid
    ));
    todo!(
      "webpack L1348: also assert merged snapshot is already cached as valid without validation; Rust has no snapshot-identity validation cache"
    );
  }

  #[tokio::test]
  async fn resolves_the_start_time_when_only_one_snapshot_has_one() {
    let fs_info = create_fs_info(create_fs());
    let no_start = fs_info
      .create_snapshot(
        None,
        &paths(FILES),
        &paths(&[]),
        &paths(&[]),
        SnapshotStrategyOptions::timestamp(),
      )
      .await
      .expect("snapshot without start time");
    let with_start = snapshot_paths(
      &fs_info,
      &[],
      DIRECTORIES,
      &[],
      SnapshotStrategyOptions::timestamp(),
    )
    .await;
    assert!(no_start.start_time.is_none());
    assert_eq!(
      fs_info
        .merge_snapshots(no_start.clone(), with_start.clone())
        .start_time,
      with_start.start_time
    );
    assert_eq!(
      fs_info
        .merge_snapshots(with_start.clone(), no_start.clone())
        .start_time,
      with_start.start_time
    );
    assert!(
      fs_info
        .merge_snapshots(no_start.clone(), no_start)
        .start_time
        .is_none()
    );
  }
}

// webpack: "snapshot optimization" (L1410).
mod snapshot_optimization {
  use super::*;

  #[tokio::test]
  #[should_panic] // We don't support snapshot optimization yet
  async fn reuses_a_whole_shared_snapshot_and_splits_it_on_partial_overlap() {
    let fs = create_fs();
    let fs_info = create_fs_info(fs.clone());
    let mode = SnapshotStrategyOptions::timestamp();
    snapshot_paths(&fs_info, FILES, &[], &[], mode).await;
    snapshot_paths(&fs_info, FILES, &[], &[], mode).await;
    let third = snapshot_paths(&fs_info, FILES, &[], &[], mode).await;
    // Observe the shared child itself instead of webpack's JS-only counters.
    // Keep this assertion red until SnapshotOptimization is implemented.
    assert!(
      third
        .children
        .as_ref()
        .is_some_and(|children| !children.is_empty()),
      "the third identical snapshot should reference a shared child"
    );
    let partial = snapshot_paths(&fs_info, &FILES[..FILES.len() - 2], &[], &[], mode).await;
    assert!(
      partial
        .children
        .as_ref()
        .is_some_and(|children| !children.is_empty()),
      "partial overlap should retain a shared child"
    );
    expect_snapshot_state(fs, &partial, true).await;
  }
}

// webpack: "resolveBuildDependencies" (L1450).
mod resolve_build_dependencies {
  use super::*;

  fn create_project_fs() -> Arc<TestFileSystem> {
    let fs = Arc::new(TestFileSystem::new());
    fs.mkdir("/proj/empty-dir");
    fs.write("/proj/package.json", r#"{"name":"proj","version":"1.0.0","dependencies":{"dep":"1.0.0"},"optionalDependencies":{"missing-dep":"1.0.0"}}"#);
    fs.write(
      "/proj/entry.js",
      "import \"./lib.mjs\";\nimport \"dep\";\nimport(\"./dyn.mjs\");\n",
    );
    fs.write("/proj/lib.mjs", "export const a = 1;\n");
    fs.write("/proj/dyn.mjs", "export const b = 2;\n");
    fs.write(
      "/proj/node_modules/dep/package.json",
      r#"{"name":"dep","version":"1.0.0","main":"index.js"}"#,
    );
    fs.write("/proj/node_modules/dep/index.js", "module.exports = 1;");
    fs
  }

  fn create_project_fs_info(fs: Arc<TestFileSystem>) -> FileSystemInfo {
    let mut options = options();
    options.managed_paths.clear();
    options.unmanaged_paths.clear();
    options.immutable_paths.clear();
    fs_info_with_options(fs, options).0
  }

  #[tokio::test]
  #[should_panic(expected = "not yet implemented: webpack L1515-L1531:")]
  async fn collects_files_directories_and_resolve_results_across_cjs_esm() {
    let fs_info = create_project_fs_info(create_project_fs());
    let result = fs_info
      .resolve_build_dependencies(
        paths(&["/proj/entry.js", "/proj/", "/proj/empty-dir/"]).into_iter(),
      )
      .await;
    for path in [
      "/proj/entry.js",
      "/proj/lib.mjs",
      "/proj/node_modules/dep/index.js",
    ] {
      assert!(
        result.files.contains(&InternedPath::from(path)),
        "missing build dependency {path}"
      );
    }
    todo!(
      "webpack L1515-L1531: ResolvedBuildDependencies lacks resolveDependencies.missing (/proj/empty-dir/package.json), resolveResults (including false for missing-dep), and checkResolveResultsValid"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1537: resolve /proj/, create node_modules/missing-dep/package.json, revalidate old results as false; resolveResults and checkResolveResultsValid are missing"
  )]
  fn check_resolve_results_valid_reports_invalid_when_an_expected_missing_dep_appears() {
    todo!(
      "{}",
      "webpack L1537: resolve /proj/, create node_modules/missing-dep/package.json, revalidate old results as false; resolveResults and checkResolveResultsValid are missing"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1564: checkResolveResultsValid must reject key x\\n/proj\\n./entry; no resolve-result type or validation API exists yet"
  )]
  fn check_resolve_results_valid_errors_on_an_unexpected_key_type() {
    todo!(
      "{}",
      "webpack L1564: checkResolveResultsValid must reject key x\\n/proj\\n./entry; no resolve-result type or validation API exists yet"
    );
  }

  #[tokio::test]
  async fn parses_esm_specifiers_covering_every_string_escape_form() {
    let fs = Arc::new(TestFileSystem::new());
    // Keep even invalid escapes: webpack tolerates them while tracing build
    // dependencies. This upstream case asserts completion, not a resolved graph.
    let source = [
      "import(``);".to_string(),
      r"import(`./plain.mjs`);".into(),
      r"import(`./hex\x41.mjs`);".into(),
      r"import(`./unicode\u0041.mjs`);".into(),
      r"import(`./codepoint\u{1F600}.mjs`);".into(),
      r"import(`./named\n\t\r\b\f\v.mjs`);".into(),
      r"import(`./nul\0.mjs`);".into(),
      r"import(`./other\q\$.mjs`);".into(),
      "import(`./cont\\\nlf.mjs`);".into(),
      "import(`./cont\\\rcr.mjs`);".into(),
      "import(`./cont\\\r\ncrlf.mjs`);".into(),
      "import(`./cont\\\u{2028}ls.mjs`);".into(),
      "import(`./cont\\\u{2029}ps.mjs`);".into(),
      "import(`./raw\rcr.mjs`);".into(),
      r"import(`./bad-hex\xZZ.mjs`);".into(),
      r"import(`./bad-unicode\uZZZZ.mjs`);".into(),
      r"import(`./bad-codepoint\u{110000}.mjs`);".into(),
      r"import(`./empty-codepoint\u{}.mjs`);".into(),
      r"import(`./octal\101.mjs`);".into(),
      r"import(`./decimal\8.mjs`);".into(),
      r#"import("\101" + x);"#.into(),
      r#"import("\8" + x);"#.into(),
      r#"import(x + "\u0041");"#.into(),
    ]
    .join("\n")
      + "\n";
    fs.write("/proj/entry.mjs", source);
    let fs_info = create_project_fs_info(fs);
    let result = fs_info
      .resolve_build_dependencies(paths(&["/proj/entry.mjs"]).into_iter())
      .await;
    assert!(
      result
        .files
        .contains(&InternedPath::from("/proj/entry.mjs"))
    );
  }
}

// webpack: "managed item info" (L1626).
mod managed_item_info {
  use super::*;

  fn setup_managed(extra: impl FnOnce(&TestFileSystem)) -> (FileSystemInfo, Arc<TestLog>) {
    let fs = Arc::new(TestFileSystem::new());
    fs.write(
      "/root/node_modules/normal/package.json",
      r#"{"name":"normal","version":"1.0.0"}"#,
    );
    extra(&fs);
    let mut options = options();
    options.managed_paths = vec![PathMatcher::String("/root/node_modules".into())];
    options.unmanaged_paths.clear();
    options.immutable_paths.clear();
    fs_info_with_options(fs, options)
  }

  #[tokio::test]
  async fn captures_a_normal_package_and_a_nested_grouping_folder() {
    let (fs_info, _) = setup_managed(|fs| fs.mkdir("/root/node_modules/group/node_modules"));
    let snapshot = snapshot_paths(
      &fs_info,
      &[
        "/root/node_modules/normal/index.js",
        "/root/node_modules/group/index.js",
      ],
      &[],
      &[],
      SnapshotStrategyOptions::timestamp(),
    )
    .await;
    let info = snapshot.managed_item_info.expect("managed info");
    assert_eq!(
      info
        .get(&InternedPath::from("/root/node_modules/normal"))
        .map(String::as_str),
      Some("normal@1.0.0")
    );
    assert_eq!(
      info
        .get(&InternedPath::from("/root/node_modules/group"))
        .map(String::as_str),
      Some("*nested")
    );
  }

  #[tokio::test]
  async fn treats_a_nested_node_modules_directory_as_node_modules() {
    let (fs_info, _) = setup_managed(|fs| fs.mkdir("/root/node_modules/sub/node_modules"));
    let snapshot = snapshot_paths(
      &fs_info,
      &["/root/node_modules/sub/node_modules"],
      &[],
      &[],
      SnapshotStrategyOptions::timestamp(),
    )
    .await;
    let info = snapshot.managed_item_info.expect("managed info");
    assert_eq!(
      info
        .get(&InternedPath::from("/root/node_modules/sub/node_modules"))
        .map(String::as_str),
      Some("*node_modules")
    );
  }

  #[tokio::test]
  async fn warns_on_a_package_json_without_a_name_and_on_a_non_package_folder() {
    let (fs_info, log) = setup_managed(|fs| {
      fs.write(
        "/root/node_modules/noname/package.json",
        r#"{"version":"1.0.0"}"#,
      );
      fs.write("/root/node_modules/bare/file.txt", "x");
    });
    snapshot_paths(
      &fs_info,
      &[
        "/root/node_modules/noname/index.js",
        "/root/node_modules/bare/index.js",
      ],
      &[],
      &[],
      SnapshotStrategyOptions::timestamp(),
    )
    .await;
    let warnings = log.0.lock().expect("log lock");
    assert!(
      warnings
        .iter()
        .any(|message| message.contains("doesn't contain a \"name\""))
    );
    assert!(
      warnings
        .iter()
        .any(|message| message.contains("isn't a directory or doesn't contain a package.json"))
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1756: malformed /root/node_modules/broken/package.json aborts with a null snapshot; create_snapshot returns Result<Snapshot>, so the successful no-snapshot outcome is not representable yet"
  )]
  fn propagates_a_package_json_json_parse_error() {
    todo!(
      "{}",
      "webpack L1756: malformed /root/node_modules/broken/package.json aborts with a null snapshot; create_snapshot returns Result<Snapshot>, so the successful no-snapshot outcome is not representable yet"
    );
  }
}

// webpack: "cached getters" (L1779).
mod cached_getters {
  use super::*;

  #[tokio::test]
  async fn serve_resolved_values_from_the_cache() {
    let fs = create_fs();
    let fs_info = create_fs_info(fs.clone());
    for mode in [
      SnapshotStrategyOptions::timestamp(),
      SnapshotStrategyOptions::hash(),
      SnapshotStrategyOptions::hash_and_timestamp(),
    ] {
      snapshot(&fs_info, mode).await;
    }
    let before = fs.calls();
    let file = InternedPath::from("/path/file.txt");
    let dir = InternedPath::from("/path/context");
    assert!(
      fs_info
        .file_timestamp(&file)
        .await
        .expect("timestamp")
        .is_some()
    );
    assert!(matches!(
      fs_info.file_hash(&file).await.expect("file hash"),
      Some(FileHash::Digest(_))
    ));
    assert!(
      fs_info
        .context_timestamp(&dir)
        .await
        .expect("context timestamp")
        .is_some()
    );
    assert!(
      fs_info
        .context_hash(&dir)
        .await
        .expect("context hash")
        .is_some()
    );
    assert!(
      fs_info
        .context_timestamp_and_hash(&dir)
        .await
        .expect("context tsh")
        .is_some()
    );
    assert_eq!(
      fs.calls(),
      before,
      "cached getters must not read the filesystem again"
    );
  }
}

// webpack: "serialization and iteration" (L1841).
mod serialization_and_iteration {
  use super::*;

  fn assert_fields_equal(original: &Snapshot, restored: &Snapshot) {
    assert_eq!(original.start_time, restored.start_time, "start_time");
    assert_eq!(
      original.file_timestamps, restored.file_timestamps,
      "file_timestamps"
    );
    assert_eq!(original.file_hashes, restored.file_hashes, "file_hashes");
    assert_eq!(
      original.file_timestamp_hashes, restored.file_timestamp_hashes,
      "file_timestamp_hashes"
    );
    assert_eq!(
      original.context_timestamps, restored.context_timestamps,
      "context_timestamps"
    );
    assert_eq!(
      original.context_hashes, restored.context_hashes,
      "context_hashes"
    );
    assert_eq!(
      original.context_timestamp_hashes, restored.context_timestamp_hashes,
      "context_timestamp_hashes"
    );
    assert_eq!(
      original.missing_existence, restored.missing_existence,
      "missing_existence"
    );
    assert_eq!(
      original.managed_item_info, restored.managed_item_info,
      "managed_item_info"
    );
    assert_eq!(
      original.managed_files, restored.managed_files,
      "managed_files"
    );
    assert_eq!(
      original.managed_contexts, restored.managed_contexts,
      "managed_contexts"
    );
    assert_eq!(
      original.managed_missing, restored.managed_missing,
      "managed_missing"
    );
    assert_eq!(original.children.is_some(), restored.children.is_some());
    if let Some(children) = &original.children {
      let restored_children = restored.children.as_ref().expect("restored children");
      assert_eq!(children.len(), restored_children.len());
      for (original, restored) in children.iter().zip(restored_children) {
        assert_fields_equal(original, restored);
      }
    }
  }

  #[tokio::test]
  #[should_panic(expected = "not yet implemented: webpack L1913-L1926:")]
  async fn round_trips_every_snapshot_field_and_iterates_across_children() {
    let fs_info = create_fs_info(create_fs());
    let child_a = snapshot_paths(
      &fs_info,
      &["/path/file.txt"],
      &[],
      &[],
      SnapshotStrategyOptions::timestamp(),
    )
    .await;
    let child_b = snapshot_paths(
      &fs_info,
      &["/path/file2.txt"],
      &[],
      &[],
      SnapshotStrategyOptions::timestamp(),
    )
    .await;
    let mut base = snapshot(&fs_info, SnapshotStrategyOptions::hash_and_timestamp()).await;
    base.file_timestamps = Some([(InternedPath::from("/f"), None)].into_iter().collect());
    base.file_hashes = Some(
      [(
        InternedPath::from("/f"),
        Some(FileHash::Digest(fs_info.digest(b"h"))),
      )]
      .into_iter()
      .collect(),
    );
    base.context_timestamps = Some([(InternedPath::from("/c"), None)].into_iter().collect());
    base.context_hashes = Some(
      [(InternedPath::from("/c"), Some(fs_info.digest(b"h")))]
        .into_iter()
        .collect(),
    );
    base.managed_item_info = Some(
      [(InternedPath::from("/m"), "m@1.0.0".to_string())]
        .into_iter()
        .collect(),
    );
    base.managed_files = Some(paths(&["/mf"]));
    base.managed_contexts = Some(paths(&["/mc"]));
    base.managed_missing = Some(paths(&["/mm"]));
    base.children = Some(vec![child_a, child_b]);
    let restored = round_trip(&base);
    assert_fields_equal(&base, &restored);
    assert!(restored.start_time.is_some());
    assert!(restored.file_timestamps.is_some());
    assert!(restored.file_hashes.is_some());
    assert!(restored.file_timestamp_hashes.is_some());
    assert!(restored.context_timestamps.is_some());
    assert!(restored.context_hashes.is_some());
    assert!(restored.context_timestamp_hashes.is_some());
    assert!(restored.missing_existence.is_some());
    assert!(restored.managed_item_info.is_some());
    assert!(restored.managed_files.is_some());
    assert!(restored.managed_contexts.is_some());
    assert!(restored.managed_missing.is_some());
    assert_eq!(restored.children.as_ref().expect("children").len(), 2);
    todo!(
      "webpack L1913-L1926: remaining assertions need Snapshot file/context/missing iterables across multiple children and the single-child shortcut; serialization assertions above already exercise every Rust field"
    );
  }
}

// webpack: "read errors abort snapshot creation" (L1936).
mod read_errors_abort_snapshot_creation {
  use super::*;

  #[tokio::test]
  #[should_panic(expected = "not yet implemented: webpack L1937:")]
  async fn hash_mode_stores_directory_for_a_directory_and_aborts_on_read_error() {
    let fs = create_fs();
    let fs_info = create_fs_info(fs.clone());
    assert_eq!(
      fs_info
        .file_hash(&InternedPath::from("/path/context"))
        .await
        .expect("directory sentinel"),
      Some(FileHash::Directory)
    );
    fs.fail(
      Operation::Read,
      "/path/file.txt",
      ErrorKind::PermissionDenied,
      usize::MAX,
    );
    todo!(
      "webpack L1937: create_snapshot(hash) must abort with a null snapshot after EACCES; Rust returns Result<Snapshot> and cannot return successful no-snapshot yet"
    );
  }

  #[test]
  #[should_panic(
    expected = "webpack L1965: inject stat EACCES for /path/file.txt and require a null snapshot; create_snapshot currently propagates Err instead of supporting successful no-snapshot"
  )]
  fn timestamp_mode_aborts_when_stat_fails_with_a_non_enoent_error() {
    todo!(
      "{}",
      "webpack L1965: inject stat EACCES for /path/file.txt and require a null snapshot; create_snapshot currently propagates Err instead of supporting successful no-snapshot"
    );
  }
}
