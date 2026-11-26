use std::{
  fs,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsWatcher, FsWatcherIgnored, FsWatcherOptions, TimeInfoEntry,
};
use tokio::sync::{
  mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
  oneshot,
};

type Event = rspack_error::Result<String>;

struct Changes(UnboundedSender<Event>);

impl EventHandler for Changes {
  fn on_change(&self, path: String) -> rspack_error::Result<()> {
    let _ = self.0.send(Ok(path));
    Ok(())
  }
}

impl EventAggregateHandler for Changes {
  fn on_event_handle(&self, _: FxHashSet<String>, _: FxHashSet<String>) {}

  fn on_error(&self, error: rspack_error::Error) {
    let _ = self.0.send(Err(error));
  }
}

async fn watch(
  watcher: &FsWatcher,
  contexts: &[&Path],
  added_missing: &[&Path],
  removed_missing: &[&Path],
) -> UnboundedReceiver<Event> {
  let (tx, rx) = unbounded_channel();
  watcher
    .watch(
      (std::iter::empty(), std::iter::empty()),
      (
        contexts.iter().copied().map(InternedPath::from),
        std::iter::empty(),
      ),
      (
        added_missing.iter().copied().map(InternedPath::from),
        removed_missing.iter().copied().map(InternedPath::from),
      ),
      SystemTime::now(),
      Box::new(Changes(tx.clone())),
      Box::new(Changes(tx)),
    )
    .await;
  rx
}

async fn wait_for(rx: &mut UnboundedReceiver<Event>, path: &Path) {
  tokio::time::timeout(Duration::from_secs(5), async {
    while let Some(changed) = rx.recv().await {
      if Path::new(&changed.expect("watch must succeed")) == path {
        return;
      }
    }
    panic!("event channel closed before {}", path.display());
  })
  .await
  .unwrap_or_else(|_| panic!("no disk event for {}", path.display()));
}

fn pause_at(
  path: PathBuf,
) -> (
  FsWatcherIgnored,
  impl FnOnce() -> (oneshot::Receiver<()>, oneshot::Sender<()>),
) {
  let (entered, waiting) = oneshot::channel();
  let (release, resume) = oneshot::channel();
  let gate = Arc::new(Mutex::new(
    None::<(oneshot::Sender<()>, oneshot::Receiver<()>)>,
  ));
  let arm = {
    let gate = Arc::clone(&gate);
    move || {
      *gate.lock().expect("arm gate") = Some((entered, resume));
      (waiting, release)
    }
  };
  let ignored = FsWatcherIgnored::Function(Arc::new(move |entry| {
    let pause = if Path::new(&entry) == path {
      gate.lock().expect("lock gate").take()
    } else {
      None
    };
    Box::pin(async move {
      if let Some((entered, resume)) = pause {
        entered.send(()).expect("signal paused observation");
        resume.await.expect("resume observation");
      }
      false
    })
  }));
  (ignored, arm)
}

fn fixture() -> (tempfile::TempDir, PathBuf, FsWatcher) {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(20),
      ..Default::default()
    },
    Default::default(),
  );
  (temp, root, watcher)
}

#[tokio::test]
async fn recursive_watches_stay_within_the_requested_trees() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let left = InternedPath::from(root.join("left"));
  let right = InternedPath::from(root.join("right"));
  fs::create_dir(&left).expect("create left tree");
  fs::create_dir(&right).expect("create right tree");
  let child = InternedPath::from(left.join("child.js"));
  let other = InternedPath::from(right.join("other.js"));
  let analyzer = rspack_watcher::testing::DirectoryAnalyzer::default();
  analyzer
    .update(
      (vec![child, other], vec![]),
      (vec![left.clone()], vec![]),
      (vec![], vec![]),
    )
    .await
    .expect("register both trees");
  let expected: FxHashSet<_> = [left.clone(), right].into_iter().collect();
  assert_eq!(analyzer.recursive_roots(), expected);
  analyzer
    .update(
      (vec![], vec![]),
      (vec![], vec![left.clone()]),
      (vec![], vec![]),
    )
    .await
    .expect("remove only the left context");
  assert_eq!(analyzer.recursive_roots(), expected);
}

#[tokio::test]
async fn disjoint_roots_and_shared_directories_keep_their_surviving_subscriptions() {
  let (_temp, root, watcher) = fixture();
  let left = root.join("left");
  let right = root.join("right");
  fs::create_dir(&left).expect("create first tree");
  fs::create_dir(&right).expect("create second tree");
  let first = left.join("first.js");
  let second = left.join("second.js");
  let other = right.join("other.js");
  let mut events = watch(&watcher, &[], &[&first, &second, &other], &[]).await;
  fs::write(&first, "first").expect("write first tree");
  wait_for(&mut events, &first).await;
  fs::write(&other, "other").expect("write second tree");
  wait_for(&mut events, &other).await;

  let mut events = watch(&watcher, &[], &[], &[&first, &other]).await;
  fs::write(&second, "second").expect("write surviving subscription");
  wait_for(&mut events, &second).await;
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn removing_a_recursive_root_keeps_a_registered_child_listening() {
  let (_temp, root, watcher) = fixture();
  let kept = root.join("kept");
  fs::create_dir(&kept).expect("create registered child's directory");
  let child = kept.join("child.js");
  let deep = root.join("missing/deep.js");
  let _events = watch(&watcher, &[], &[&child, &deep], &[]).await;

  let mut events = watch(&watcher, &[], &[], &[&deep]).await;
  fs::write(&child, "child").expect("write registered child");
  wait_for(&mut events, &child).await;
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn a_recovered_context_observes_its_contents_and_later_writes() {
  let (_temp, root, watcher) = fixture();
  let context = root.join("context");
  let staged = root.join("staged");
  fs::create_dir(&staged).expect("create staged directory");
  fs::write(staged.join("before.js"), "before").expect("populate staged directory");
  let mut events = watch(&watcher, &[&context], &[], &[]).await;

  fs::rename(staged, &context).expect("restore context");
  wait_for(&mut events, &context).await;
  let before = context.join("before.js");
  let (files, _) = watcher.collect_time_info_entries();
  assert!(
    files.iter().any(
      |(name, entry)| Path::new(name) == before && matches!(entry, TimeInfoEntry::Entry { .. })
    ),
    "recovery must scan the moved-in contents: {files:?}"
  );
  let after = context.join("after.js");
  fs::write(&after, "after").expect("write after recovery scan");
  wait_for(&mut events, &after).await;
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn registration_and_recovery_watch_writes_while_the_scan_is_paused() {
  for restore in [false, true] {
    let temp = tempfile::tempdir().expect("create fixture");
    let root = temp.path().canonicalize().expect("resolve fixture");
    let context = root.join("context");
    let source = if restore {
      root.join("staged")
    } else {
      context.clone()
    };
    fs::create_dir(&source).expect("create directory");
    fs::write(source.join("marker"), "seed").expect("seed scan");

    let (ignored, arm) = pause_at(context.join("marker"));
    let (waiting, release) = arm();
    let watcher = FsWatcher::new(Default::default(), ignored);
    let transient = context.join("transient.js");
    let write_during_scan = async {
      tokio::time::timeout(Duration::from_secs(5), waiting)
        .await
        .expect("scan must reach marker")
        .expect("enter scan");
      fs::write(&transient, "temporary").expect("write during scan");
      fs::remove_file(&transient).expect("remove before scanning resumes");
      release.send(()).expect("resume scan");
    };
    let contexts = [context.as_path()];
    let registration = watch(&watcher, &contexts, &[], &[]);
    let mut events = if restore {
      let events = registration.await;
      fs::rename(source, &context).expect("restore directory");
      write_during_scan.await;
      events
    } else {
      tokio::join!(registration, write_during_scan).0
    };
    wait_for(&mut events, &transient).await;
    watcher.close().await.expect("close watcher");
  }
}

#[cfg(unix)]
#[tokio::test]
async fn a_searchable_parent_does_not_prevent_watching_a_readable_child() {
  use std::os::unix::fs::PermissionsExt;
  let (_temp, root, watcher) = fixture();
  let parent = root.join("search-only");
  let child = parent.join("readable");
  fs::create_dir_all(&child).expect("create readable child directory");
  let file = child.join("new.js");
  let permissions = fs::metadata(&parent).expect("stat parent").permissions();
  fs::set_permissions(&parent, fs::Permissions::from_mode(0o111)).expect("restrict parent listing");
  let mut events = watch(&watcher, &[], &[&file], &[]).await;
  fs::set_permissions(&parent, permissions).expect("restore fixture permissions");
  fs::write(&file, "new").expect("write in readable child");
  wait_for(&mut events, &file).await;
  watcher.close().await.expect("close watcher");
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn removing_disjoint_roots_keeps_the_fsevents_stream_for_live_writes() {
  let (_temp, root, watcher) = fixture();
  let kept = root.join("kept");
  fs::create_dir(&kept).expect("create surviving context");
  let others: Vec<_> = (0..40)
    .map(|n| {
      let dir = root.join(n.to_string());
      fs::create_dir(&dir).expect("create independent directory");
      dir.join("other.js")
    })
    .collect();
  let others: Vec<_> = others.iter().map(PathBuf::as_path).collect();
  let _events = watch(&watcher, &[&kept], &others, &[]).await;
  let expected: Vec<_> = (0..80).map(|n| kept.join(format!("{n}.js"))).collect();
  let writes = expected.clone();
  let (started, writing) = tokio::sync::oneshot::channel();
  let writer = std::thread::spawn(move || {
    started.send(()).expect("signal writer");
    for path in writes {
      fs::write(path, "changed").expect("write surviving tree");
      std::thread::sleep(Duration::from_millis(2));
    }
  });
  writing.await.expect("writer running");
  let _events = watch(&watcher, &[], &[], &others).await;
  writer.join().expect("finish writes");
  tokio::time::timeout(Duration::from_secs(5), async {
    loop {
      let (files, _) = watcher.collect_time_info_entries();
      if expected
        .iter()
        .all(|path| files.iter().any(|(name, _)| Path::new(name) == path))
      {
        break;
      }
      tokio::time::sleep(Duration::from_millis(20)).await;
    }
  })
  .await
  .expect("every write in the surviving tree must reach the snapshot");
  watcher.close().await.expect("close watcher");
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
#[tokio::test]
async fn a_reported_root_removal_releases_the_old_kernel_watch() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let context = root.join("context");
  fs::create_dir(&context).expect("create context");
  let (ignored, arm) = pause_at(context.clone());
  let watcher = FsWatcher::new(Default::default(), ignored);
  let mut events = watch(&watcher, &[&context], &[], &[]).await;
  let (waiting, release) = arm();
  fs::remove_dir(&context).expect("remove watched directory");
  tokio::time::timeout(Duration::from_secs(5), waiting)
    .await
    .expect("receive root removal")
    .expect("enter gate");
  fs::create_dir(&context).expect("recreate watched directory");
  release.send(()).expect("resume processing the removal");
  wait_for(&mut events, &context).await;
  let child = context.join("after.js");
  fs::write(&child, "new inode").expect("write restored tree");
  wait_for(&mut events, &child).await;
  watcher.close().await.expect("close watcher");
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn an_unreadable_sibling_does_not_prevent_recovery_of_a_missing_branch() {
  use std::os::unix::fs::PermissionsExt;
  let (_temp, root, watcher) = fixture();
  let sibling = root.join("private");
  fs::create_dir(&sibling).expect("create unrelated sibling");
  let permissions = fs::metadata(&sibling).expect("stat sibling").permissions();
  fs::set_permissions(&sibling, fs::Permissions::from_mode(0o000)).expect("restrict sibling");
  let target = root.join("branch/nested/file.js");
  let mut events = watch(&watcher, &[], &[&target], &[]).await;
  fs::set_permissions(&sibling, permissions).expect("restore fixture permissions");
  fs::create_dir_all(target.parent().expect("target parent")).expect("create missing directories");
  fs::write(&target, "restored").expect("create missing dependency");
  wait_for(&mut events, &target).await;
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn a_parent_watch_only_checks_entries_needed_for_recovery() {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let project = root.join("project");
  fs::create_dir(&project).expect("create watched directory");
  let sibling = root.join("unrelated.js");
  let file = project.join("file.js");
  let (ignored, arm) = pause_at(sibling.clone());
  let watcher = FsWatcher::new(Default::default(), ignored);
  let mut events = watch(&watcher, &[], &[&file], &[]).await;
  let (mut called, release) = arm();
  fs::write(sibling, "unrelated").expect("write outside the watched root");
  fs::write(&file, "changed").expect("write watched file");
  let checked_sibling = tokio::select! {
    _ = &mut called => true,
    () = wait_for(&mut events, &file) => tokio::time::timeout(Duration::from_millis(250), &mut called).await.is_ok(),
  };
  let _ = release.send(());
  assert!(
    !checked_sibling,
    "an unrelated sibling reached the user-supplied ignored function"
  );
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn a_context_appearing_during_its_ancestor_walk_is_scanned_before_delivery() {
  use notify::{Event, EventKind, event::CreateKind};
  use rspack_watcher::{FsEventKind, testing::DirectoryAnalyzer};

  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  let parent = root.join("parent");
  let branch = parent.join("branch");
  let context = InternedPath::from(branch.join("context"));
  let staged = root.join("staged");
  fs::create_dir_all(&branch).expect("create parent tree");
  fs::create_dir(&staged).expect("create staged context");
  fs::write(staged.join("before.js"), "before").expect("populate staged context");

  let (ignored, arm) = pause_at(branch);
  let mut analyzer = DirectoryAnalyzer::with_ignored(ignored);
  analyzer
    .update(
      (vec![], vec![]),
      (vec![context.clone()], vec![]),
      (vec![], vec![]),
    )
    .await
    .expect("register absent context");
  let (waiting, release) = arm();
  analyzer.on_native_event(Event::new(EventKind::Create(CreateKind::Folder)).add_path(parent));
  tokio::time::timeout(Duration::from_secs(5), waiting)
    .await
    .expect("walk must reach the branch")
    .expect("enter walk");
  fs::rename(staged, &context).expect("restore context while walking");
  release.send(()).expect("resume walk");

  let changes = tokio::time::timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .expect("report the restored context");
  assert!(changes.contains(&(context.clone(), FsEventKind::Create)));
  let (files, _) = analyzer.collect_time_info_entries();
  assert!(
    files
      .iter()
      .any(|(path, entry)| Path::new(path) == context.join("before.js")
        && matches!(entry, TimeInfoEntry::Entry { .. })),
    "restored directory contents must be present when its change is delivered: {files:?}"
  );
}
