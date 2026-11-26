mod scan_support;

use std::fs;

use notify::{EventKind, event::ModifyKind};
use rspack_paths::InternedPath;
use rspack_watcher::{TimeInfoEntry, testing::DirectoryAnalyzer};
use scan_support::{added, empty, entry, notify, update};

#[tokio::test]
async fn a_live_file_replacing_a_directory_forgets_its_descendants() {
  let fixture = tempfile::tempdir().unwrap();
  let root = fixture.path().canonicalize().unwrap();
  let child = root.join("entry");
  let old = child.join("old.js");
  fs::create_dir(&child).unwrap();
  fs::write(&old, "").unwrap();
  let mut analyzer = DirectoryAnalyzer::default();
  update(&analyzer, empty(), added(&root), empty()).await;
  analyzer.scan_contexts().await.unwrap();
  let (before, _) = analyzer.collect_time_info_entries();
  assert!(matches!(
    entry(&before, &old),
    Some(TimeInfoEntry::Entry { .. })
  ));

  fs::remove_dir_all(&child).unwrap();
  fs::write(&child, "now a file").unwrap();
  notify(&mut analyzer, &child, EventKind::Modify(ModifyKind::Any)).await;
  let (files, directories) = analyzer.collect_time_info_entries();
  assert_eq!(entry(&directories, &child), None);
  assert_eq!(entry(&files, &old), None);
  assert!(matches!(
    entry(&files, &child),
    Some(TimeInfoEntry::Entry { .. })
  ));

  fs::remove_file(&child).unwrap();
  fs::create_dir(&child).unwrap();
  let fresh = child.join("fresh.js");
  fs::write(&fresh, "").unwrap();
  notify(&mut analyzer, &child, EventKind::Modify(ModifyKind::Any)).await;
  let (files, directories) = analyzer.collect_time_info_entries();
  assert!(matches!(
    entry(&directories, &child),
    Some(TimeInfoEntry::OnlySafeTimeEntry { .. })
  ));
  assert_eq!(
    entry(&files, &child),
    Some(&TimeInfoEntry::ExistenceOnlyTimeEntry)
  );
  assert!(matches!(
    entry(&files, &fresh),
    Some(TimeInfoEntry::Entry { .. })
  ));
}

#[tokio::test]
async fn replacing_an_explicit_context_with_a_file_forgets_its_old_children() {
  for parent_watched in [false, true] {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().canonicalize().unwrap();
    let child = root.join("nested");
    let old = child.join("old.js");
    fs::create_dir(&child).unwrap();
    fs::write(&old, "").unwrap();
    let mut analyzer = DirectoryAnalyzer::default();
    let mut contexts = added(&child);
    if parent_watched {
      contexts.0.push(InternedPath::from(&root));
    }
    update(&analyzer, empty(), contexts, empty()).await;
    analyzer.scan_contexts().await.unwrap();
    assert!(entry(&analyzer.collect_time_info_entries().0, &old).is_some());
    fs::remove_dir_all(&child).unwrap();
    fs::write(&child, "now a file").unwrap();
    notify(&mut analyzer, &child, EventKind::Modify(ModifyKind::Any)).await;
    assert_eq!(entry(&analyzer.collect_time_info_entries().0, &old), None);
    fs::remove_file(&child).unwrap();
    fs::create_dir(&child).unwrap();
    let fresh = child.join("fresh.js");
    fs::write(&fresh, "").unwrap();
    notify(&mut analyzer, &child, EventKind::Modify(ModifyKind::Any)).await;
    assert!(matches!(
      entry(&analyzer.collect_time_info_entries().0, &fresh),
      Some(TimeInfoEntry::Entry { .. })
    ));
  }
}
