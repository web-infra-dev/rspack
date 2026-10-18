use std::{
  path::Path,
  sync::Arc,
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use rspack_util::time::system_time_to_millis;

use crate::{
  FsEventKind, TimeInfoEntry,
  dirwatch::{WatcherManager, disk_mtime},
  state::{Origin, PathDiff},
};

fn diff(added: &[&InternedPath], removed: &[&InternedPath]) -> PathDiff {
  (
    added.iter().map(|p| (*p).clone()).collect(),
    removed.iter().map(|p| (*p).clone()).collect(),
  )
}

fn write_old(path: impl AsRef<Path>) {
  let path = path.as_ref();
  std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
  std::fs::write(path, "entry").expect("write");
  std::fs::File::options()
    .write(true)
    .open(path)
    .expect("open")
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(123))
    .expect("mtime");
}

async fn watch(
  manager: &Arc<WatcherManager>,
  files: PathDiff,
  contexts: PathDiff,
  missing: PathDiff,
) {
  manager.reset();
  manager
    .update(files, contexts, missing)
    .await
    .expect("register");
  let added: Vec<_> = manager
    .registry
    .files
    .added
    .iter()
    .map(|p| p.clone())
    .collect();
  for path in added {
    if let Some(mtime) = disk_mtime(&path) {
      manager.set_file_time_if_absent(path, mtime);
    }
  }
  manager.scan_contexts(SystemTime::now()).await;
  manager.scan_registered(SystemTime::now()).await;
}

fn file_entry(manager: &WatcherManager, path: &InternedPath) -> Option<TimeInfoEntry> {
  manager
    .collect_time_info_entries()
    .0
    .into_iter()
    .find(|(key, _)| key == &path.to_string_lossy())
    .map(|(_, entry)| entry)
}

fn directory_entry(manager: &WatcherManager, path: &InternedPath) -> Option<TimeInfoEntry> {
  manager
    .collect_time_info_entries()
    .1
    .into_iter()
    .find(|(key, _)| key == &path.to_string_lossy())
    .map(|(_, entry)| entry)
}

fn assert_file_time(manager: &WatcherManager, path: &InternedPath) -> u64 {
  let TimeInfoEntry::Entry {
    timestamp,
    safe_time,
    accuracy,
  } = file_entry(manager, path).expect("file time")
  else {
    panic!("expected file timestamp for {path:?}");
  };
  assert_eq!(
    timestamp,
    system_time_to_millis(disk_mtime(path).expect("mtime"))
  );
  assert!(safe_time >= timestamp);
  assert!(accuracy <= 2000);
  safe_time
}

#[tokio::test]
async fn initial_file_timestamps_survive_later_subscription_cycles() {
  let dir = tempfile::tempdir().expect("directory");
  let file = InternedPath::from(dir.path().join("entry.js"));
  std::fs::write(&file, "entry").expect("file");
  let initial = system_time_to_millis(disk_mtime(&file).expect("mtime"));
  let state = WatcherManager::default();
  state
    .update(diff(&[&file], &[]), Default::default(), Default::default())
    .await
    .expect("register");
  state.record_initial_file_mtimes();
  write_old(&file);
  state.reset();
  state
    .update(Default::default(), Default::default(), Default::default())
    .await
    .expect("next cycle");
  state.record_initial_file_mtimes();
  let (files, directories) = state.collect_time_info_entries();
  assert!(directories.is_empty());
  assert_eq!(files.len(), 1);
  assert!(matches!(files[0].1, TimeInfoEntry::Entry { timestamp, .. } if timestamp == initial));
}

#[tokio::test]
async fn mixed_tree_reports_recorded_files_missing_paths_and_directory_safe_times() {
  let dir = tempfile::tempdir().expect("directory");
  let at = |name: &str| InternedPath::from(dir.path().join(name));
  let (file, gone, late, missing) = (
    at("d/file.js"),
    at("d/gone.js"),
    at("d/late.js"),
    at("d/missing.js"),
  );
  let (context, absent, registered, found_missing) =
    (at("ctx"), at("absent"), at("ctx/reg.js"), at("ctx/m.js"));
  for path in [
    &file,
    &registered,
    &found_missing,
    &at("ctx/a.js"),
    &at("ctx/sub/b.js"),
    &at("ctx/sub/deep/c.js"),
  ] {
    write_old(path);
  }
  std::fs::create_dir_all(at("ctx/empty")).expect("empty directory");
  let manager = Arc::new(WatcherManager::default());
  watch(
    &manager,
    diff(&[&file, &gone, &late, &registered], &[]),
    diff(&[&context, &absent], &[]),
    diff(&[&missing, &found_missing], &[]),
  )
  .await;
  write_old(&late);
  for path in [
    &file,
    &late,
    &registered,
    &found_missing,
    &at("ctx/a.js"),
    &at("ctx/sub/b.js"),
    &at("ctx/sub/deep/c.js"),
  ] {
    assert_file_time(&manager, path);
  }
  for path in [&gone, &missing, &absent] {
    assert_eq!(file_entry(&manager, path), Some(TimeInfoEntry::Null));
  }
  assert_eq!(
    directory_entry(&manager, &absent),
    Some(TimeInfoEntry::Null)
  );
  let nested_safe = assert_file_time(&manager, &at("ctx/sub/deep/c.js"));
  for path in [&context, &at("ctx/sub"), &at("ctx/sub/deep")] {
    assert_eq!(
      file_entry(&manager, path),
      Some(TimeInfoEntry::ExistenceOnlyTimeEntry)
    );
    assert_eq!(
      directory_entry(&manager, path),
      Some(TimeInfoEntry::OnlySafeTimeEntry {
        safe_time: nested_safe
      })
    );
  }
  assert_eq!(
    directory_entry(&manager, &at("ctx/empty")),
    Some(TimeInfoEntry::OnlySafeTimeEntry { safe_time: 0 })
  );
  let (files, directories) = manager.collect_time_info_entries();
  assert_eq!(files.len(), 15);
  assert_eq!(directories.len(), 5);
}

#[tokio::test]
async fn backfilled_missing_is_not_retained_as_a_discovered_file() {
  let dir = tempfile::tempdir().expect("directory");
  let context = InternedPath::from(dir.path().join("ctx"));
  let missing = InternedPath::from(context.join("missing.js"));
  write_old(context.join("other.js"));
  let manager = Arc::new(WatcherManager::default());
  watch(
    &manager,
    Default::default(),
    diff(&[&context], &[]),
    Default::default(),
  )
  .await;
  write_old(&missing);
  watch(
    &manager,
    Default::default(),
    Default::default(),
    diff(&[&missing], &[]),
  )
  .await;
  assert_file_time(&manager, &missing);
  assert_eq!(
    manager
      .collect_time_info_entries()
      .0
      .iter()
      .filter(|(key, _)| key == &missing.to_string_lossy())
      .count(),
    1
  );
  watch(
    &manager,
    Default::default(),
    Default::default(),
    diff(&[], &[&missing]),
  )
  .await;
  assert_eq!(file_entry(&manager, &missing), None);
}

#[tokio::test]
async fn seeded_file_is_not_retained_as_a_discovered_file() {
  let dir = tempfile::tempdir().expect("directory");
  let context = InternedPath::from(dir.path().join("ctx"));
  let file = InternedPath::from(context.join("file.js"));
  write_old(context.join("other.js"));
  let manager = Arc::new(WatcherManager::default());
  watch(
    &manager,
    Default::default(),
    diff(&[&context], &[]),
    Default::default(),
  )
  .await;
  write_old(&file);
  watch(
    &manager,
    diff(&[&file], &[]),
    Default::default(),
    Default::default(),
  )
  .await;
  assert_file_time(&manager, &file);
  watch(
    &manager,
    diff(&[], &[&file]),
    Default::default(),
    Default::default(),
  )
  .await;
  assert_eq!(file_entry(&manager, &file), None);
}

#[tokio::test]
async fn registered_missing_paths_are_not_stated_by_time_info_collection() {
  let dir = tempfile::tempdir().expect("directory");
  let missing: Vec<_> = (0..10)
    .map(|i| InternedPath::from(dir.path().join(format!("m{i}.js"))))
    .collect();
  let manager = WatcherManager::default();
  manager
    .update(
      Default::default(),
      Default::default(),
      (missing.clone(), vec![]),
    )
    .await
    .expect("register");
  for path in missing {
    write_old(path);
  }
  let (files, directories) = manager.collect_time_info_entries();
  assert_eq!(files.len(), 10);
  assert!(files.iter().all(|(_, entry)| *entry == TimeInfoEntry::Null));
  assert!(directories.is_empty());
}

#[tokio::test]
async fn deleting_context_before_scan_events_reports_removal_and_null_times() {
  let dir = tempfile::tempdir().expect("directory");
  let context = InternedPath::from(dir.path().join("ctx"));
  let sub = InternedPath::from(context.join("sub"));
  write_old(sub.join("file.js"));
  let manager = Arc::new(WatcherManager::default());
  watch(
    &manager,
    Default::default(),
    diff(&[&context], &[]),
    Default::default(),
  )
  .await;
  std::fs::remove_dir_all(&context).expect("remove context");
  for path in [&sub, &context] {
    assert_eq!(
      manager
        .process(path, FsEventKind::Change, Origin::Scan)
        .await,
      Some(vec![(context.clone(), FsEventKind::Remove)])
    );
  }
  let batches = manager.scan_registered(SystemTime::now()).await;
  let events: Vec<_> = batches
    .into_iter()
    .flatten()
    .map(|e| (e.path, e.kind))
    .collect();
  assert_eq!(events, vec![(context.clone(), FsEventKind::Remove)]);
  assert_eq!(
    manager.collect_time_info_entries(),
    (
      vec![(context.to_string_lossy().to_string(), TimeInfoEntry::Null)],
      vec![(context.to_string_lossy().to_string(), TimeInfoEntry::Null)]
    )
  );
}

#[tokio::test]
async fn events_and_subscription_changes_preserve_time_info() {
  let dir = tempfile::tempdir().expect("directory");
  let at = |name: &str| InternedPath::from(dir.path().join(name));
  let (file, moved, context_file) = (at("d/file.js"), at("d/moved.js"), at("ctx/reg.js"));
  let (context, inner, absent) = (at("ctx"), at("ctx/sub/deep"), at("absent"));
  let (missing, context_missing) = (at("d/missing.js"), at("ctx/missing.js"));
  for path in [
    &file,
    &moved,
    &context_file,
    &at("ctx/a.js"),
    &at("ctx/sub/deep/c.js"),
    &at("ctx/keep/k.js"),
  ] {
    write_old(path);
  }
  let manager = Arc::new(WatcherManager::default());
  watch(
    &manager,
    diff(&[&file, &moved, &context_file], &[]),
    diff(&[&context, &inner, &absent], &[]),
    diff(&[&missing, &context_missing], &[]),
  )
  .await;
  for (path, kind, expected) in [
    (at("ctx/a.js"), FsEventKind::Change, vec![context.clone()]),
    (
      at("ctx/sub/new.js"),
      FsEventKind::Create,
      vec![context.clone()],
    ),
    (
      context_missing.clone(),
      FsEventKind::Change,
      vec![context_missing.clone(), context.clone()],
    ),
    (missing.clone(), FsEventKind::Create, vec![missing.clone()]),
    (file.clone(), FsEventKind::Change, vec![file.clone()]),
  ] {
    std::fs::write(&path, "changed").expect("change file");
    let before = crate::state::now();
    let events = manager
      .process(
        &path,
        kind,
        Origin::Disk {
          appeared: kind == FsEventKind::Create,
        },
      )
      .await
      .expect("events");
    assert_eq!(events.len(), expected.len());
    for expected in expected {
      let expected_kind = if expected == path {
        kind
      } else {
        FsEventKind::Change
      };
      assert!(events.contains(&(expected, expected_kind)));
    }
    assert!(assert_file_time(&manager, &path) >= before);
  }
  write_old(at("ctx/newdir/inside.js"));
  manager
    .process(
      &at("ctx/newdir"),
      FsEventKind::Create,
      Origin::Disk { appeared: true },
    )
    .await;
  assert_file_time(&manager, &at("ctx/newdir/inside.js"));
  write_old(at("absent/back.js"));
  manager
    .process(
      &absent,
      FsEventKind::Create,
      Origin::Disk { appeared: true },
    )
    .await;
  assert_file_time(&manager, &at("absent/back.js"));
  std::fs::remove_file(&file).expect("remove file");
  assert_eq!(
    manager
      .process(&file, FsEventKind::Remove, Origin::Disk { appeared: false })
      .await,
    Some(vec![(file.clone(), FsEventKind::Remove)])
  );
  assert_eq!(file_entry(&manager, &file), Some(TimeInfoEntry::Null));
  std::fs::remove_dir_all(at("ctx/sub")).expect("remove nested context");
  let events = manager
    .process(
      &at("ctx/sub"),
      FsEventKind::Remove,
      Origin::Disk { appeared: false },
    )
    .await
    .expect("remove events");
  assert!(events.contains(&(inner.clone(), FsEventKind::Remove)));
  assert_eq!(directory_entry(&manager, &inner), Some(TimeInfoEntry::Null));
  let before_removal = crate::state::now();
  std::fs::remove_dir_all(&absent).expect("remove context");
  manager
    .process(
      &absent,
      FsEventKind::Remove,
      Origin::Disk { appeared: false },
    )
    .await;
  assert_eq!(
    directory_entry(&manager, &absent),
    Some(TimeInfoEntry::Null)
  );
  std::fs::create_dir(&absent).expect("recreate without event");
  let mut options = std::fs::OpenOptions::new();
  options.read(true);
  #[cfg(windows)]
  {
    use std::os::windows::fs::OpenOptionsExt;
    options.custom_flags(0x02000000);
  }
  options
    .open(&absent)
    .expect("open directory")
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(123))
    .expect("old directory mtime");
  assert!(
    matches!(directory_entry(&manager, &absent), Some(TimeInfoEntry::OnlySafeTimeEntry { safe_time }) if safe_time >= before_removal)
  );
  assert_eq!(
    file_entry(&manager, &absent),
    Some(TimeInfoEntry::ExistenceOnlyTimeEntry)
  );
  write_old(at("d/other.js"));
  assert_eq!(
    manager
      .process(
        &at("d/other.js"),
        FsEventKind::Create,
        Origin::Disk { appeared: true }
      )
      .await,
    Some(vec![])
  );
  assert_eq!(file_entry(&manager, &at("d/other.js")), None);
  let keep = at("ctx/keep");
  watch(
    &manager,
    Default::default(),
    diff(&[&keep], &[]),
    Default::default(),
  )
  .await;
  write_old(at("ctx/keep/k2.js"));
  let events = manager
    .process(
      &at("ctx/keep/k2.js"),
      FsEventKind::Create,
      Origin::Disk { appeared: true },
    )
    .await
    .expect("nested events");
  assert!(events.contains(&(keep.clone(), FsEventKind::Change)));
  watch(
    &manager,
    Default::default(),
    diff(&[], &[&keep]),
    Default::default(),
  )
  .await;
  assert_file_time(&manager, &at("ctx/keep/k2.js"));
  watch(
    &manager,
    diff(&[], &[&moved, &context_file]),
    Default::default(),
    diff(&[&moved], &[]),
  )
  .await;
  assert_file_time(&manager, &moved);
  assert_file_time(&manager, &context_file);
  watch(
    &manager,
    Default::default(),
    diff(&[], &[&context, &inner, &absent]),
    diff(&[], &[&missing, &context_missing, &moved]),
  )
  .await;
  assert_eq!(
    manager.collect_time_info_entries(),
    (
      vec![(file.to_string_lossy().to_string(), TimeInfoEntry::Null)],
      vec![]
    )
  );
}
