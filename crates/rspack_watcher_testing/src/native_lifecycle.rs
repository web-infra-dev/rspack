use std::time::{Duration, SystemTime};

use rspack_paths::InternedPath;
use tokio::sync::mpsc;

use crate::{EventAggregateHandler, EventHandler, FsEventKind, FsWatcher, FsWatcherOptions};

struct Aggregate;
impl EventAggregateHandler for Aggregate {
  fn on_event_handle(&self, _: crate::HashSet<String>, _: crate::HashSet<String>) {}
}

struct Events(mpsc::UnboundedSender<(String, FsEventKind)>);
impl EventHandler for Events {
  fn on_change(&self, path: String) -> rspack_error::Result<()> {
    let _ = self.0.send((path, FsEventKind::Change));
    Ok(())
  }
  fn on_delete(&self, path: String) -> rspack_error::Result<()> {
    let _ = self.0.send((path, FsEventKind::Remove));
    Ok(())
  }
}

fn write_file(path: &InternedPath, seconds: u64) {
  std::fs::write(path, seconds.to_string()).expect("file");
  std::fs::File::options()
    .write(true)
    .open(path)
    .expect("open")
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds))
    .expect("mtime");
}

async fn receive(
  rx: &mut mpsc::UnboundedReceiver<(String, FsEventKind)>,
  path: &InternedPath,
  kind: FsEventKind,
  phase: &str,
) {
  tokio::time::timeout(Duration::from_secs(5), async {
    while let Some(event) = rx.recv().await {
      if event == (path.to_string_lossy().into_owned(), kind) {
        return;
      }
    }
    panic!("watcher closed before {kind:?}");
  })
  .await
  .unwrap_or_else(|_| panic!("missing native {kind:?} for {path:?} during {phase}"));
}

async fn recreated_parent(delete: bool, immediate_replacement: bool) {
  let temp = tempfile::tempdir().expect("directory");
  let base = temp.path().canonicalize().expect("canonicalize");
  let parent = base.join("parent");
  std::fs::create_dir(&parent).expect("parent");
  let file = InternedPath::from(parent.join("entry.js"));
  write_file(&file, 100);
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(0),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, mut rx) = mpsc::unbounded_channel();
  watcher
    .watch(
      (vec![file.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Aggregate),
      Box::new(Events(tx)),
    )
    .await;
  write_file(&file, 101);
  receive(&mut rx, &file, FsEventKind::Change, "initial edit").await;
  let replacement = base.join("replacement");
  if immediate_replacement {
    std::fs::create_dir(&replacement).expect("prepare parent");
    write_file(&InternedPath::from(replacement.join("entry.js")), 102);
  }
  if delete {
    std::fs::remove_dir_all(&parent).expect("delete parent");
  } else {
    std::fs::rename(&parent, base.join("old-parent")).expect("move parent away");
  }
  if immediate_replacement {
    std::fs::rename(replacement, &parent).expect("replace parent");
  } else {
    receive(&mut rx, &file, FsEventKind::Remove, "parent removal").await;
    #[cfg(target_os = "linux")]
    wait_for_inotify_watch(&base, true).await;
    std::fs::create_dir(&parent).expect("recreate parent");
    write_file(&file, 102);
  }
  receive(&mut rx, &file, FsEventKind::Change, "parent recovery").await;
  tokio::time::timeout(Duration::from_secs(5), async {
    while tokio::time::timeout(Duration::from_millis(200), rx.recv())
      .await
      .is_ok()
    {}
  })
  .await
  .expect("native events settled");
  #[cfg(target_os = "linux")]
  {
    wait_for_inotify_watch(&base, false).await;
    wait_for_inotify_watch(&parent, true).await;
  }
  write_file(&file, 103);
  receive(&mut rx, &file, FsEventKind::Change, "edit after recovery").await;
  watcher.close().await.expect("close");
}

#[tokio::test]
async fn native_watcher_keeps_watching_a_recreated_parent() {
  recreated_parent(false, false).await;
}

#[tokio::test]
async fn native_watcher_keeps_watching_after_parent_deletion() {
  recreated_parent(true, false).await;
}

#[tokio::test]
async fn native_watcher_keeps_watching_an_immediately_replaced_parent() {
  recreated_parent(false, true).await;
}

#[cfg(target_os = "linux")]
fn has_inotify_watch(path: &std::path::Path) -> bool {
  use std::os::unix::fs::MetadataExt;

  let inode = std::fs::metadata(path).expect("watched directory").ino();
  std::fs::read_dir("/proc/self/fdinfo")
    .expect("fdinfo")
    .filter_map(Result::ok)
    .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
    .any(|info| {
      info
        .lines()
        .filter(|line| line.starts_with("inotify "))
        .any(|line| {
          line.split_whitespace().any(|field| {
            field
              .strip_prefix("ino:")
              .and_then(|value| u64::from_str_radix(value, 16).ok())
              == Some(inode)
          })
        })
    })
}

#[cfg(target_os = "linux")]
async fn wait_for_inotify_watch(path: &std::path::Path, expected: bool) {
  tokio::time::timeout(Duration::from_secs(5), async {
    while has_inotify_watch(path) != expected {
      tokio::time::sleep(Duration::from_millis(10)).await;
    }
  })
  .await
  .unwrap_or_else(|_| panic!("inotify watch for {path:?}: expected {expected}"));
}

#[cfg(target_os = "linux")]
async fn initially_missing_ancestors(context: bool) {
  let temp = tempfile::tempdir().expect("directory");
  let base = temp.path().canonicalize().expect("canonicalize");
  let unrelated = base.join("unrelated");
  std::fs::create_dir(&unrelated).expect("unrelated directory");
  let first = base.join("first");
  let parent = first.join("parent");
  let file = InternedPath::from(parent.join("entry.js"));
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(0),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, mut rx) = mpsc::unbounded_channel();
  watcher
    .watch(
      (std::iter::empty(), std::iter::empty()),
      (
        if context {
          vec![InternedPath::from(parent.as_path())]
        } else {
          vec![]
        }
        .into_iter(),
        std::iter::empty(),
      ),
      (
        if context { vec![] } else { vec![file.clone()] }.into_iter(),
        std::iter::empty(),
      ),
      SystemTime::now(),
      Box::new(Aggregate),
      Box::new(Events(tx)),
    )
    .await;
  wait_for_inotify_watch(&base, true).await;
  assert!(!has_inotify_watch(&unrelated), "unrelated subtree watched");
  std::fs::create_dir(&first).expect("first ancestor");
  wait_for_inotify_watch(&base, false).await;
  wait_for_inotify_watch(&first, true).await;
  std::fs::create_dir(&parent).expect("parent");
  write_file(&file, 100);
  let event_path = if context {
    InternedPath::from(parent.as_path())
  } else {
    file.clone()
  };
  receive(
    &mut rx,
    &event_path,
    FsEventKind::Change,
    "initially missing parent",
  )
  .await;
  wait_for_inotify_watch(&first, false).await;
  wait_for_inotify_watch(&parent, true).await;
  tokio::time::timeout(Duration::from_secs(5), async {
    while tokio::time::timeout(Duration::from_millis(200), rx.recv())
      .await
      .is_ok()
    {}
  })
  .await
  .expect("recovery events settled");
  write_file(&file, 101);
  receive(
    &mut rx,
    &event_path,
    FsEventKind::Change,
    "edit after narrowing",
  )
  .await;
  watcher.close().await.expect("close");
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn native_watcher_narrows_initially_missing_ancestors() {
  initially_missing_ancestors(false).await;
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn native_watcher_narrows_initially_missing_context_ancestors() {
  initially_missing_ancestors(true).await;
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn removing_a_context_after_its_child_disappears_keeps_the_child_subscription() {
  struct Errors(mpsc::UnboundedSender<String>);
  impl EventAggregateHandler for Errors {
    fn on_event_handle(&self, _: crate::HashSet<String>, _: crate::HashSet<String>) {}
    fn on_error(&self, error: rspack_error::Error) {
      let _ = self.0.send(error.to_string());
    }
  }
  let temp = tempfile::tempdir().expect("directory");
  let root = InternedPath::from(temp.path().canonicalize().expect("canonicalize"));
  let parent = root.join("nested");
  std::fs::create_dir(&parent).expect("parent");
  let file = InternedPath::from(parent.join("entry.js"));
  write_file(&file, 100);
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(0),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, mut rx) = mpsc::unbounded_channel();
  let (error_tx, mut errors) = mpsc::unbounded_channel();
  watcher
    .watch(
      (vec![file.clone()].into_iter(), std::iter::empty()),
      (vec![root.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Errors(error_tx.clone())),
      Box::new(Events(tx.clone())),
    )
    .await;
  write_file(&file, 101);
  receive(&mut rx, &file, FsEventKind::Change, "initial edit").await;
  let archive = tempfile::tempdir().expect("archive");
  std::fs::rename(&parent, archive.path().join("nested")).expect("move child away");
  receive(&mut rx, &file, FsEventKind::Remove, "child removal").await;
  tokio::time::sleep(Duration::from_millis(200)).await;
  watcher
    .watch(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), vec![root].into_iter()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now(),
      Box::new(Errors(error_tx)),
      Box::new(Events(tx)),
    )
    .await;
  let error = errors.try_recv();
  assert!(
    error.is_err(),
    "failed to transfer the child watch: {error:?}"
  );
  std::fs::create_dir(&parent).expect("restore child");
  write_file(&file, 102);
  receive(
    &mut rx,
    &file,
    FsEventKind::Change,
    "child recovery after context removal",
  )
  .await;
  watcher.close().await.expect("close");
}
