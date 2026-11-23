use std::{
  fs,
  time::{Duration, SystemTime},
};

use notify::{
  Event, EventKind,
  event::{CreateKind, ModifyKind, RemoveKind, RenameMode},
};
use rspack_paths::InternedPath;
use rspack_watcher::{FsEventKind, testing::DirectoryAnalyzer};
use tempfile::TempDir;

fn rename(mode: RenameMode, paths: &[&InternedPath]) -> Event {
  paths.iter().fold(
    Event::new(EventKind::Modify(ModifyKind::Name(mode))),
    |event, path| event.add_path(path.as_ref().to_path_buf()),
  )
}

async fn next_events(analyzer: &mut DirectoryAnalyzer) -> Vec<(InternedPath, FsEventKind)> {
  tokio::time::timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .expect("rename should report the dependency change")
}

async fn assert_file_rename(paired: bool) {
  let temp = TempDir::new().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let from = InternedPath::from(base.join("index.js"));
  let to = InternedPath::from(base.join("index.js.map"));
  fs::write(&from, "source").expect("write source");
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(
      (vec![from.clone()], vec![]),
      (vec![], vec![]),
      (vec![to.clone()], vec![]),
    )
    .await
    .expect("register dependencies");
  fs::rename(&from, &to).expect("rename file");
  if paired {
    analyzer.on_native_event(rename(RenameMode::Both, &[&from, &to]));
  } else {
    analyzer.on_native_event(rename(RenameMode::From, &[&from]));
    analyzer.on_native_event(rename(RenameMode::To, &[&to]));
  }
  assert_eq!(
    next_events(&mut analyzer).await,
    vec![(from, FsEventKind::Remove)]
  );
  assert_eq!(
    next_events(&mut analyzer).await,
    vec![(to, FsEventKind::Create)]
  );
}

#[tokio::test]
async fn test_split_rename_removes_source_and_creates_destination() {
  assert_file_rename(false).await;
}

#[tokio::test]
async fn test_paired_rename_removes_source_and_creates_destination() {
  assert_file_rename(true).await;
}

#[tokio::test]
async fn test_unknown_rename_modes_keep_change_fallback() {
  for mode in [RenameMode::Any, RenameMode::Other] {
    let temp = TempDir::new().expect("create fixture");
    let base = temp.path().canonicalize().expect("resolve fixture");
    let file = InternedPath::from(base.join("index.js"));
    fs::write(&file, "source").expect("write source");
    let mut analyzer = DirectoryAnalyzer::default();
    analyzer
      .update(
        (vec![file.clone()], vec![]),
        (vec![], vec![]),
        (vec![], vec![]),
      )
      .await
      .expect("register file");
    analyzer.on_native_event(rename(mode, &[&file]));
    assert_eq!(
      next_events(&mut analyzer).await,
      vec![(file, FsEventKind::Change)]
    );
  }
}

#[tokio::test]
async fn test_delayed_rename_round_trip_with_unchanged_mtime_is_ignored() {
  let temp = TempDir::new().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let file = InternedPath::from(base.join("index.js"));
  let backup = InternedPath::from(base.join("backup.js"));
  fs::write(&file, "source").expect("write source");
  fs::File::options()
    .write(true)
    .open(&file)
    .expect("open source")
    .set_modified(SystemTime::now() - Duration::from_secs(60))
    .expect("set old mtime");
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(
      (vec![file.clone()], vec![]),
      (vec![], vec![]),
      (vec![], vec![]),
    )
    .await
    .expect("register file");
  analyzer.on_native_event(
    Event::new(EventKind::Modify(ModifyKind::Any)).add_path(file.as_ref().to_path_buf()),
  );
  assert_eq!(
    next_events(&mut analyzer).await,
    vec![(file.clone(), FsEventKind::Change)]
  );

  let mtime = fs::metadata(&file)
    .expect("stat file")
    .modified()
    .expect("read mtime");
  fs::rename(&file, &backup).expect("rename away");
  fs::rename(&backup, &file).expect("rename back");
  assert_eq!(
    fs::metadata(&file)
      .expect("stat restored file")
      .modified()
      .expect("read restored mtime"),
    mtime
  );
  // The OS callback may run after the complete save operation.
  analyzer.on_native_event(rename(RenameMode::From, &[&file]));
  analyzer.on_native_event(rename(RenameMode::To, &[&file]));
  assert!(
    tokio::time::timeout(Duration::from_millis(200), analyzer.next_events())
      .await
      .is_err(),
    "a restored path with unchanged mtime should be ignored like watchpack"
  );
}

#[tokio::test]
async fn test_unchanged_mtime_inside_the_accuracy_window_still_reports_changes() {
  for (kind, expected) in [
    (EventKind::Remove(RemoveKind::File), FsEventKind::Change),
    (
      EventKind::Modify(ModifyKind::Name(RenameMode::From)),
      FsEventKind::Change,
    ),
    (
      EventKind::Modify(ModifyKind::Name(RenameMode::To)),
      FsEventKind::Create,
    ),
    (EventKind::Create(CreateKind::File), FsEventKind::Create),
  ] {
    let temp = TempDir::new().expect("create fixture");
    let file = InternedPath::from(
      temp
        .path()
        .canonicalize()
        .expect("resolve fixture")
        .join("index.js"),
    );
    fs::write(&file, "source").expect("write source");
    // A future mtime keeps the accuracy window open regardless of scheduling.
    fs::File::options()
      .write(true)
      .open(&file)
      .expect("open source")
      .set_modified(SystemTime::now() + Duration::from_secs(60))
      .expect("set future mtime");
    let mut analyzer = DirectoryAnalyzer::default();
    analyzer
      .update(
        (vec![file.clone()], vec![]),
        (vec![], vec![]),
        (vec![], vec![]),
      )
      .await
      .expect("register file");
    analyzer.on_native_event(
      Event::new(EventKind::Modify(ModifyKind::Any)).add_path(file.as_ref().to_path_buf()),
    );
    assert_eq!(
      next_events(&mut analyzer).await,
      vec![(file.clone(), FsEventKind::Change)]
    );
    analyzer.on_native_event(Event::new(kind).add_path(file.as_ref().to_path_buf()));
    assert_eq!(next_events(&mut analyzer).await, vec![(file, expected)]);
  }
}

#[tokio::test]
async fn test_paired_directory_rename_discovers_missing_descendant() {
  let temp = TempDir::new().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let from = InternedPath::from(base.join("staged"));
  let to = InternedPath::from(base.join("modules"));
  let child = InternedPath::from(base.join("modules/nested/index.js"));
  fs::create_dir_all(base.join("staged/nested")).expect("create staging directory");
  fs::write(base.join("staged/nested/index.js"), "source").expect("write staged file");
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(
      (vec![], vec![]),
      (vec![], vec![]),
      (vec![child.clone()], vec![]),
    )
    .await
    .expect("register missing file");
  fs::rename(&from, &to).expect("rename directory");
  analyzer.on_native_event(rename(RenameMode::Both, &[&from, &to]));
  // The source itself has no registered dependency.
  let mut events = next_events(&mut analyzer).await;
  if events.is_empty() {
    events = next_events(&mut analyzer).await;
  }
  assert!(
    events.contains(&(child, FsEventKind::Create)),
    "missing descendant was not reported: {events:?}"
  );
}
