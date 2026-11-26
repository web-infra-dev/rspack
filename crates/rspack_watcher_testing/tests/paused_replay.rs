use std::{
  path::Path,
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind, FsWatcher, FsWatcherOptions,
};
use tokio::{sync::mpsc, time::timeout};

#[derive(Debug)]
enum Delivery {
  Raw(String),
  Batch(FxHashSet<String>, FxHashSet<String>),
}

struct Handler(mpsc::UnboundedSender<Delivery>);

impl EventHandler for Handler {
  fn on_delete(&self, path: String) -> rspack_error::Result<()> {
    self
      .0
      .send(Delivery::Raw(path))
      .expect("record raw delivery");
    Ok(())
  }
}

impl EventAggregateHandler for Handler {
  fn on_event_handle(&self, changed: FxHashSet<String>, removed: FxHashSet<String>) {
    self
      .0
      .send(Delivery::Batch(changed, removed))
      .expect("record batch");
  }
}

async fn watch_missing(
  watcher: &FsWatcher,
  added: &[&Path],
  removed: &[&Path],
) -> mpsc::UnboundedReceiver<Delivery> {
  let (tx, rx) = mpsc::unbounded_channel();
  watcher
    .watch(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (
        added.iter().copied().map(InternedPath::from),
        removed.iter().copied().map(InternedPath::from),
      ),
      SystemTime::now() + Duration::from_secs(60),
      Box::new(Handler(tx.clone())),
      Box::new(Handler(tx)),
    )
    .await;
  rx
}

#[tokio::test]
async fn paused_inputs_are_delivered_to_the_next_watch_subscriptions() {
  check_replay(false).await;
}

#[tokio::test]
async fn consuming_a_paused_batch_still_leaves_inputs_without_a_recipient() {
  check_replay(true).await;
}

async fn check_replay(consume: bool) {
  let fixture = tempfile::tempdir().expect("create fixture");
  let root = fixture.path().canonicalize().expect("resolve fixture");
  let retired = root.join("retired");
  let added = root.join("added");
  let retained = root.join("retained");
  let watcher = FsWatcher::new(
    FsWatcherOptions {
      aggregate_timeout: Some(30),
      ..Default::default()
    },
    Default::default(),
  );
  let mut old = watch_missing(&watcher, &[&retired, &retained], &[]).await;
  watcher.pause().expect("pause watcher");
  for path in [&retired, &added, &retained] {
    watcher.trigger_event(&path.into(), FsEventKind::Remove);
  }
  timeout(Duration::from_secs(5), async {
    while !watcher
      .peek_aggregated()
      .1
      .contains(retained.to_str().expect("UTF-8 fixture path"))
    {
      tokio::task::yield_now().await;
    }
  })
  .await
  .expect("old subscriptions can inspect pending removals");
  if consume {
    assert_eq!(watcher.take_aggregated().1.len(), 2);
  }
  let mut current = watch_missing(&watcher, &[&added], &[&retired]).await;
  let (raw, changed, removed) = timeout(Duration::from_secs(5), async {
    let mut raw = Vec::new();
    loop {
      match current.recv().await.expect("receive callback") {
        Delivery::Raw(path) => raw.push(path),
        Delivery::Batch(changed, removed) => return (raw, changed, removed),
      }
    }
  })
  .await
  .expect("replay the newly subscribed path");
  watcher.close().await.expect("close watcher");
  assert!(
    old.try_recv().is_err(),
    "the paused handler received a callback"
  );
  let mut expected = vec![added.to_str().expect("UTF-8 fixture path").to_owned()];
  if !consume {
    expected.push(retained.to_str().expect("UTF-8 fixture path").to_owned());
  }
  assert_eq!(raw, expected);
  assert!(changed.is_empty());
  assert_eq!(removed, expected.into_iter().collect::<FxHashSet<_>>());
}

#[tokio::test]
async fn a_second_pause_during_replay_keeps_the_first_consumption() {
  consume_during_replay(true).await;
}

#[tokio::test]
async fn pending_inputs_can_be_consumed_while_replay_waits_on_ignored() {
  consume_during_replay(false).await;
}

async fn consume_during_replay(pause_again: bool) {
  use std::sync::{Arc, Mutex};

  use tokio::sync::oneshot;
  let fixture = tempfile::tempdir().expect("create fixture");
  let root = fixture.path().canonicalize().expect("resolve fixture");
  let file = root.join("pending");
  let marker = root.join("marker");
  let (entered, waiting) = oneshot::channel();
  let (release, resume) = oneshot::channel();
  let gate = Arc::new(Mutex::new(None));
  let ignored = rspack_watcher::FsWatcherIgnored::Function(Arc::new({
    let file = file.clone();
    let gate = Arc::clone(&gate);
    move |path| {
      let block = if Path::new(&path) == file {
        gate.lock().expect("lock replay gate").take()
      } else {
        None
      };
      Box::pin(async move {
        if let Some((entered, resume)) = block {
          let entered: oneshot::Sender<()> = entered;
          entered.send(()).expect("enter replay");
          let resume: oneshot::Receiver<()> = resume;
          let _ = resume.await;
        }
        false
      })
    }
  }));
  let watcher = FsWatcher::new(Default::default(), ignored);
  let _old = watch_missing(&watcher, &[&file, &marker], &[]).await;
  watcher.pause().expect("pause watcher");
  watcher.trigger_event(&(&file).into(), FsEventKind::Remove);
  timeout(Duration::from_secs(5), async {
    while !watcher
      .peek_aggregated()
      .1
      .contains(file.to_str().expect("UTF-8 fixture path"))
    {
      tokio::task::yield_now().await;
    }
  })
  .await
  .expect("deliver pending removals before timeout");
  if pause_again {
    assert!(
      watcher
        .take_aggregated()
        .1
        .contains(file.to_str().expect("UTF-8 fixture path"))
    );
  }
  *gate.lock().expect("lock replay gate") = Some((entered, resume));
  let mut current = watch_missing(&watcher, &[], &[]).await;
  timeout(Duration::from_secs(5), waiting)
    .await
    .expect("replay reached ignored before timeout")
    .expect("gate signaled");
  if pause_again {
    watcher.pause().expect("pause watcher");
  } else {
    assert!(
      watcher
        .take_aggregated()
        .1
        .contains(file.to_str().expect("UTF-8 fixture path")),
      "pending removal disappeared during replay"
    );
  }
  release.send(()).expect("release replay");
  if pause_again {
    current = watch_missing(&watcher, &[], &[]).await;
  }
  watcher.trigger_event(&(&marker).into(), FsEventKind::Remove);
  timeout(Duration::from_secs(5), async {
    loop {
      match current.recv().await.expect("receive callback") {
        Delivery::Raw(path) => assert_ne!(Path::new(&path), file),
        Delivery::Batch(_, removed) => {
          assert!(
            !removed.contains(file.to_str().expect("UTF-8 fixture path")),
            "removal consumed in an earlier pause"
          );
          if removed.contains(marker.to_str().expect("UTF-8 fixture path")) {
            break;
          }
        }
      }
    }
  })
  .await
  .expect("deliver pending removals before timeout");
  watcher.close().await.expect("close watcher");
}
