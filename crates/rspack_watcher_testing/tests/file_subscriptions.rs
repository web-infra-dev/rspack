use std::{
  fs,
  io::Write,
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind, TimeInfoEntry,
  testing::{check_subscriptions, disk_event, manual_watcher, watch_file_subscriptions},
};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

struct Changes(UnboundedSender<String>);

impl EventHandler for Changes {
  fn on_change(&self, path: String) -> rspack_error::Result<()> {
    self.0.send(path).expect("collect change");
    Ok(())
  }

  fn on_delete(&self, path: String) -> rspack_error::Result<()> {
    self
      .0
      .send(format!("removed {path}"))
      .expect("collect removal");
    Ok(())
  }
}

struct Batches(UnboundedSender<(FxHashSet<String>, FxHashSet<String>)>);

impl EventAggregateHandler for Batches {
  fn on_event_handle(&self, changed: FxHashSet<String>, removed: FxHashSet<String>) {
    self.0.send((changed, removed)).expect("collect batch");
  }
}

fn write_version(path: &InternedPath, version: u64) {
  let mut file = fs::File::create(path).expect("open observed file");
  writeln!(file, "{version}").expect("write observed file");
  file
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1_600_000_000 + version))
    .expect("set distinct, aged mtime");
}

#[tokio::test]
async fn one_disk_event_notifies_both_file_subscribers() {
  assert_shared_file(false).await;
}

#[tokio::test]
async fn one_scan_notifies_both_file_subscribers() {
  assert_shared_file(true).await;
}

async fn assert_shared_file(scan: bool) {
  let temp = tempfile::tempdir().expect("create fixture");
  let root = temp.path().canonicalize().expect("resolve fixture");
  fs::create_dir(root.join("disk")).expect("create observed directory");
  let source = InternedPath::from(root.join("disk/shared.js"));
  let a = InternedPath::from(root.join("app/a.js"));
  let b = InternedPath::from(root.join("app/b.js"));
  let watcher = manual_watcher();
  let (raw_tx, mut raw) = unbounded_channel();
  let (batch_tx, mut batches) = unbounded_channel();
  write_version(&source, 0);

  for (version, added, removed, expected) in [
    (1, vec![a.clone(), b.clone()], vec![], vec![&a, &b]),
    (2, vec![], vec![a.clone()], vec![&b]),
    (3, vec![], vec![b.clone()], vec![]),
  ] {
    let bind = |paths: Vec<InternedPath>| {
      paths
        .into_iter()
        .map(|p| (source.clone(), p))
        .collect::<Vec<_>>()
    };
    watch_file_subscriptions(
      &watcher,
      &bind(added),
      &bind(removed),
      Batches(batch_tx.clone()),
      Changes(raw_tx.clone()),
    )
    .await;
    write_version(&source, version);
    if scan {
      check_subscriptions(&watcher).await;
    } else {
      disk_event(&watcher, &source, FsEventKind::Change);
    }

    if expected.is_empty() {
      assert!(
        tokio::time::timeout(Duration::from_millis(200), batches.recv())
          .await
          .is_err()
      );
      assert!(raw.try_recv().is_err());
      break;
    }
    let (changed, removed) = tokio::time::timeout(Duration::from_secs(5), batches.recv())
      .await
      .expect("receive batch")
      .expect("batch handler still active");
    let expected = expected
      .into_iter()
      .map(|p| p.to_string_lossy().into_owned())
      .collect::<Vec<_>>();
    let mut immediate = vec![];
    while let Ok(path) = raw.try_recv() {
      immediate.push(path);
    }
    immediate.sort();
    assert_eq!(immediate, expected);
    assert_eq!(changed, expected.iter().cloned().collect::<FxHashSet<_>>());
    assert!(removed.is_empty());
    let (times, _) = watcher.collect_time_info_entries();
    assert_eq!(times.len(), expected.len());
    assert!(
      times.iter().all(
        |(path, entry)| expected.contains(path) && matches!(entry, TimeInfoEntry::Entry { .. })
      )
    );
    let snapshot = times;
    check_subscriptions(&watcher).await;
    assert!(
      tokio::time::timeout(Duration::from_millis(200), batches.recv())
        .await
        .is_err()
    );
    assert!(raw.try_recv().is_err());
    assert_eq!(watcher.collect_time_info_entries().0, snapshot);
  }
  watcher.close().await.expect("close watcher");
}

#[tokio::test]
async fn missing_descendant_reports_only_the_registered_file() {
  for scan in [false, true] {
    for staged in [false, true] {
      let temp = tempfile::tempdir().expect("create fixture");
      let root = temp.path().canonicalize().expect("resolve fixture");
      let parent = InternedPath::from(root.join("missing"));
      let pkg = InternedPath::from(root.join("missing/pkg"));
      let file = InternedPath::from(root.join("missing/pkg/index.js"));
      let expected = file.to_string_lossy().into_owned();
      let watcher = manual_watcher();
      let (raw_tx, mut raw) = unbounded_channel();
      let (batch_tx, mut batches) = unbounded_channel();
      let watch = |added: Vec<InternedPath>| {
        watcher.watch(
          (std::iter::empty(), std::iter::empty()),
          (std::iter::empty(), std::iter::empty()),
          (added.into_iter(), std::iter::empty()),
          SystemTime::now(),
          Box::new(Batches(batch_tx.clone())),
          Box::new(Changes(raw_tx.clone())),
        )
      };
      watch(vec![file.clone()]).await;
      let observe = async |path: &InternedPath| {
        if scan {
          check_subscriptions(&watcher).await;
        } else {
          disk_event(&watcher, path, FsEventKind::Create);
        }
      };
      check_subscriptions(&watcher).await;
      assert_eq!(
        watcher.collect_time_info_entries().0,
        vec![(expected.clone(), TimeInfoEntry::Null)]
      );
      if staged {
        for dir in [&parent, &pkg] {
          fs::create_dir(dir).expect("create missing parent");
          observe(dir).await;
          assert!(
            tokio::time::timeout(Duration::from_millis(150), batches.recv())
              .await
              .is_err()
          );
          assert!(raw.try_recv().is_err());
        }
        write_version(&file, 1);
        observe(&file).await;
      } else {
        fs::create_dir_all(&pkg).expect("install subtree");
        write_version(&file, 1);
        if scan {
          observe(&parent).await;
        } else {
          disk_event(&watcher, &parent, FsEventKind::Change);
        }
      }
      let (changed, removed) = tokio::time::timeout(Duration::from_secs(5), batches.recv())
        .await
        .expect("receive appearance")
        .expect("batch handler active");
      assert_eq!(
        changed,
        [expected.clone()].into_iter().collect::<FxHashSet<_>>()
      );
      assert!(removed.is_empty());
      assert_eq!(raw.try_recv().expect("concrete change"), expected);
      assert!(raw.try_recv().is_err());
      assert!(
        matches!(watcher.collect_time_info_entries().0.as_slice(), [(name, TimeInfoEntry::Entry { .. })] if name == &expected)
      );
      watch(vec![]).await;
      check_subscriptions(&watcher).await;
      assert!(
        tokio::time::timeout(Duration::from_millis(150), batches.recv())
          .await
          .is_err()
      );
      assert!(raw.try_recv().is_err());
      watcher.close().await.expect("close watcher");
    }
  }
}

#[tokio::test]
async fn removing_the_file_subscription_leaves_its_context_active() {
  let temp = tempfile::tempdir().expect("create fixture");
  let dir = InternedPath::from(temp.path().canonicalize().expect("resolve fixture"));
  let file = InternedPath::from(dir.join("child.js"));
  write_version(&file, 0);
  let watcher = manual_watcher();
  let (raw_tx, mut raw) = unbounded_channel();
  let (tx, mut batches) = unbounded_channel();
  for version in 1..=2 {
    let add = version == 1;
    watcher
      .watch(
        (
          add.then(|| file.clone()).into_iter(),
          (!add).then(|| file.clone()).into_iter(),
        ),
        (add.then(|| dir.clone()).into_iter(), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
        SystemTime::now() + Duration::from_secs(30),
        Box::new(Batches(tx.clone())),
        Box::new(Changes(raw_tx.clone())),
      )
      .await;
    let parent_mtime = fs::metadata(&dir)
      .expect("stat parent")
      .modified()
      .expect("parent mtime");
    write_version(&file, version);
    assert_eq!(
      fs::metadata(&dir)
        .expect("stat parent")
        .modified()
        .expect("parent mtime"),
      parent_mtime
    );
    check_subscriptions(&watcher).await;
    let (changed, removed) = tokio::time::timeout(Duration::from_secs(5), batches.recv())
      .await
      .expect("receive change")
      .expect("batch handler active");
    let file_name = file.to_string_lossy().into_owned();
    let mut expected = FxHashSet::default();
    expected.insert(dir.to_string_lossy().into_owned());
    if add {
      expected.insert(file_name.clone());
    }
    assert_eq!(changed, expected);
    assert!(removed.is_empty());
    assert_eq!(raw.try_recv().expect("concrete file event"), file_name);
    assert!(raw.try_recv().is_err());
  }
  watcher.close().await.expect("close watcher");
}
