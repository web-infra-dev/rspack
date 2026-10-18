use std::time::{Duration, SystemTime};

use rspack_paths::InternedPath;
use tokio::sync::mpsc;

use crate::{EventAggregateHandler, EventHandler, FsWatcher, FsWatcherOptions};

struct Aggregate;
impl EventAggregateHandler for Aggregate {
  fn on_event_handle(&self, _: crate::HashSet<String>, _: crate::HashSet<String>) {}
}
struct Changes(mpsc::UnboundedSender<String>);
impl EventHandler for Changes {
  fn on_change(&self, path: String) -> rspack_error::Result<()> {
    let _ = self.0.send(path);
    Ok(())
  }
  fn on_delete(&self, path: String) -> rspack_error::Result<()> {
    let _ = self.0.send(path);
    Ok(())
  }
}

fn set_mtime(path: &InternedPath, millis: u64) {
  std::fs::File::options()
    .write(true)
    .open(path)
    .expect("file")
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_millis(millis))
    .expect("mtime");
}

#[tokio::test]
async fn polling_does_not_deliver_native_events_between_scans() {
  let temp = tempfile::tempdir().expect("directory");
  let file = InternedPath::from(
    temp
      .path()
      .canonicalize()
      .expect("canonicalize")
      .join("entry.js"),
  );
  std::fs::write(&file, "entry").expect("file");
  set_mtime(&file, 100_100);
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      poll_interval: Some(60_000),
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
      Box::new(Changes(tx)),
    )
    .await;
  set_mtime(&file, 100_200);
  let event = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;
  watcher.close().await.expect("close");
  assert!(
    event.is_err(),
    "polling must wait for its scan instead of using native events: {event:?}"
  );
}

#[tokio::test]
async fn polling_reports_subsecond_and_backdated_changes_and_stops_on_close() {
  let temp = tempfile::tempdir().expect("directory");
  let file = InternedPath::from(temp.path().join("entry.js"));
  std::fs::write(&file, "entry").expect("file");
  set_mtime(&file, 100_100);
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      poll_interval: Some(20),
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
      Box::new(Changes(tx)),
    )
    .await;
  for millis in [100_200, 99_100] {
    set_mtime(&file, millis);
    let event = tokio::time::timeout(Duration::from_secs(2), rx.recv())
      .await
      .expect("polled change")
      .expect("event");
    assert_eq!(event, file.to_string_lossy());
    assert!(
      tokio::time::timeout(Duration::from_millis(100), rx.recv())
        .await
        .is_err(),
      "stable scans must be silent"
    );
  }
  std::fs::remove_file(&file).expect("remove");
  assert_eq!(
    tokio::time::timeout(Duration::from_secs(2), rx.recv())
      .await
      .expect("removal")
      .expect("event"),
    file.to_string_lossy()
  );
  assert!(
    tokio::time::timeout(Duration::from_millis(100), rx.recv())
      .await
      .is_err()
  );
  watcher.close().await.expect("close");
  std::fs::write(&file, "after close").expect("file");
  assert!(rx.recv().await.is_none(), "closing must drop the handler");
}

#[tokio::test]
async fn polling_discovers_context_children_and_missing_paths() {
  let temp = tempfile::tempdir().expect("directory");
  let context = InternedPath::from(temp.path().join("context"));
  let missing = InternedPath::from(temp.path().join("missing.js"));
  std::fs::create_dir(&context).expect("context");
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      poll_interval: Some(20),
      aggregate_timeout: Some(0),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, mut rx) = mpsc::unbounded_channel();
  watcher
    .watch(
      (std::iter::empty(), std::iter::empty()),
      (vec![context.clone()].into_iter(), std::iter::empty()),
      (vec![missing.clone()].into_iter(), std::iter::empty()),
      SystemTime::now() + Duration::from_secs(3600),
      Box::new(Aggregate),
      Box::new(Changes(tx)),
    )
    .await;
  std::fs::write(context.join("entry.js"), "entry").expect("context file");
  std::fs::write(&missing, "found").expect("missing file");
  let mut expected = crate::HashSet::from_iter([
    context.to_string_lossy().into_owned(),
    missing.to_string_lossy().into_owned(),
  ]);
  tokio::time::timeout(Duration::from_secs(2), async {
    while !expected.is_empty() {
      expected.remove(&rx.recv().await.expect("event"));
    }
  })
  .await
  .expect("context and missing changes");
  watcher.close().await.expect("close");
}

#[tokio::test]
async fn polling_preserves_pending_changes_across_subscription_updates() {
  let temp = tempfile::tempdir().expect("directory");
  let file = InternedPath::from(temp.path().join("entry.js"));
  std::fs::write(&file, "entry").expect("file");
  set_mtime(&file, 100_100);
  let registry = crate::registry::Registry::default();
  let mut polling = crate::polling::PollingWatcher::default();
  registry
    .update(
      (vec![file.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (vec![file.clone()].into_iter(), std::iter::empty()),
    )
    .await
    .expect("register");
  polling.update(registry.access());
  registry.reset();
  set_mtime(&file, 100_200);
  registry
    .update(
      (std::iter::empty(), vec![file.clone()].into_iter()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
    )
    .await
    .expect("remove file subscription");
  polling.update(registry.access());
  assert!(
    polling.scan().contains(&file),
    "remaining missing subscription must retain the old snapshot"
  );
  assert!(polling.scan().is_empty());
  registry.reset();
  registry
    .update(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), vec![file.clone()].into_iter()),
    )
    .await
    .expect("remove missing subscription");
  polling.update(registry.access());
  set_mtime(&file, 100_300);
  assert!(
    polling.scan().is_empty(),
    "unsubscribed paths must stop polling"
  );
}

async fn close_pending_poll(queue_watch: bool) {
  use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  };

  use tokio::sync::Notify;

  let temp = tempfile::tempdir().expect("directory");
  let context = InternedPath::from(temp.path());
  let file = context.join("entry.js");
  std::fs::write(&file, "entry").expect("file");
  let armed = Arc::new(AtomicBool::new(false));
  let entered = Arc::new(Notify::new());
  let release = Arc::new(Notify::new());
  let ignored = crate::FsWatcherIgnored::Function(Arc::new({
    let armed = Arc::clone(&armed);
    let entered = Arc::clone(&entered);
    let release = Arc::clone(&release);
    move |path| {
      let block = std::path::Path::new(&path) == file.as_ref() as &std::path::Path
        && armed.load(Ordering::SeqCst);
      let entered = Arc::clone(&entered);
      let release = Arc::clone(&release);
      Box::pin(async move {
        if block {
          entered.notify_one();
          release.notified().await;
        }
        false
      })
    }
  }));
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      poll_interval: Some(20),
      ..Default::default()
    },
    ignored,
  );
  let (tx, _rx) = mpsc::unbounded_channel();
  watcher
    .watch(
      (std::iter::empty(), std::iter::empty()),
      (vec![context].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now() + Duration::from_secs(3600),
      Box::new(Aggregate),
      Box::new(Changes(tx)),
    )
    .await;
  armed.store(true, Ordering::SeqCst);
  tokio::time::timeout(Duration::from_secs(2), entered.notified())
    .await
    .expect("poll reached ignored");
  let (tx, _rx) = mpsc::unbounded_channel();
  let rewatch = watcher.watch(
    (std::iter::empty(), std::iter::empty()),
    (std::iter::empty(), std::iter::empty()),
    (std::iter::empty(), std::iter::empty()),
    SystemTime::now(),
    Box::new(Aggregate),
    Box::new(Changes(tx)),
  );
  tokio::pin!(rewatch);
  if queue_watch {
    assert!(
      tokio::time::timeout(Duration::from_millis(20), &mut rewatch)
        .await
        .is_err()
    );
  }
  let closed = tokio::time::timeout(Duration::from_millis(300), watcher.close()).await;
  armed.store(false, Ordering::SeqCst);
  release.notify_waiters();
  watcher.close().await.expect("cleanup");
  if queue_watch {
    rewatch.await;
  }
  assert!(closed.is_ok(), "close must cancel a pending poll");
}

#[tokio::test]
async fn closing_cancels_a_poll_waiting_for_ignored() {
  close_pending_poll(false).await;
}

#[tokio::test]
async fn closing_cancels_a_poll_with_a_watch_request_queued() {
  close_pending_poll(true).await;
}

struct TypedChanges(mpsc::UnboundedSender<(String, crate::FsEventKind)>);
impl EventHandler for TypedChanges {
  fn on_change(&self, path: String) -> rspack_error::Result<()> {
    let _ = self.0.send((path, crate::FsEventKind::Change));
    Ok(())
  }
  fn on_delete(&self, path: String) -> rspack_error::Result<()> {
    let _ = self.0.send((path, crate::FsEventKind::Remove));
    Ok(())
  }
}

#[tokio::test]
async fn polling_recovers_an_empty_context_after_removal_and_type_changes() {
  let temp = tempfile::tempdir().expect("directory");
  let context = InternedPath::from(temp.path().join("context"));
  std::fs::create_dir(&context).expect("context");
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      poll_interval: Some(20),
      aggregate_timeout: Some(0),
      ..Default::default()
    },
    Default::default(),
  );
  let (tx, mut rx) = mpsc::unbounded_channel();
  watcher
    .watch(
      (std::iter::empty(), std::iter::empty()),
      (vec![context.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      SystemTime::now() + Duration::from_secs(3600),
      Box::new(Aggregate),
      Box::new(TypedChanges(tx)),
    )
    .await;
  for step in 0..4 {
    match step {
      0 => std::fs::remove_dir(&context).expect("remove context"),
      1 | 3 => {
        if step == 3 {
          std::fs::remove_file(&context).expect("remove replacement");
        }
        std::fs::create_dir(&context).expect("recover context");
      }
      _ => {
        std::fs::remove_dir(&context).expect("remove context");
        tokio::time::sleep(Duration::from_millis(100)).await;
        let replacement = temp.path().join("replacement");
        std::fs::write(&replacement, "replacement").expect("prepare file");
        std::fs::rename(replacement, &context).expect("replace with file");
      }
    }
    let expected = if step == 0 {
      crate::FsEventKind::Remove
    } else {
      crate::FsEventKind::Change
    };
    tokio::time::timeout(Duration::from_secs(2), async {
      loop {
        let (path, kind) = rx.recv().await.expect("event");
        assert_eq!(path, context.to_string_lossy());
        if kind == expected {
          break;
        }
        assert!(
          step >= 2 && kind == crate::FsEventKind::Remove,
          "unexpected intermediate event at step {step}: {kind:?}"
        );
      }
    })
    .await
    .expect("final context transition");
    assert!(
      tokio::time::timeout(Duration::from_millis(100), rx.recv())
        .await
        .is_err(),
      "stable context must be silent after step {step}"
    );
  }
  watcher.close().await.expect("close");
}

#[cfg(unix)]
#[tokio::test]
async fn polling_follows_directory_links_only_when_enabled() {
  for follow_symlinks in [false, true] {
    let temp = tempfile::tempdir().expect("directory");
    let context = InternedPath::from(temp.path().join("context"));
    let target = temp.path().join("target");
    std::fs::create_dir(&context).expect("context");
    std::fs::create_dir(&target).expect("target");
    let file = InternedPath::from(target.join("entry.js"));
    std::fs::write(&file, "entry").expect("file");
    set_mtime(&file, 100_100);
    std::os::unix::fs::symlink(&target, context.join("link")).expect("link");
    let watcher = FsWatcher::new(
      FsWatcherOptions {
        follow_symlinks,
        poll_interval: Some(20),
        aggregate_timeout: Some(0),
      },
      Default::default(),
    );
    let (tx, mut rx) = mpsc::unbounded_channel();
    watcher
      .watch(
        (std::iter::empty(), std::iter::empty()),
        (vec![context.clone()].into_iter(), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
        SystemTime::now() + Duration::from_secs(3600),
        Box::new(Aggregate),
        Box::new(Changes(tx)),
      )
      .await;
    set_mtime(&file, 100_200);
    let wait = if follow_symlinks {
      Duration::from_secs(2)
    } else {
      Duration::from_millis(150)
    };
    let result = tokio::time::timeout(wait, rx.recv()).await;
    watcher.close().await.expect("close");
    if follow_symlinks {
      assert_eq!(
        result.expect("target change").expect("event"),
        context.to_string_lossy()
      );
    } else {
      assert!(
        result.is_err(),
        "unfollowed directory target must not notify"
      );
    }
  }
}
