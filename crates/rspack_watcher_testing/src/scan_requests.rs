use std::{
  sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  },
  time::{Duration, SystemTime},
};

use rspack_paths::InternedPath;
use tokio::{sync::Notify, time::timeout};

use crate::{FsWatcherIgnored, dirwatch::WatcherManager};

async fn fixture() -> (
  tempfile::TempDir,
  Arc<WatcherManager>,
  InternedPath,
  Arc<Notify>,
  Arc<AtomicUsize>,
) {
  fixture_pausing(1).await
}

async fn fixture_pausing(
  pauses: usize,
) -> (
  tempfile::TempDir,
  Arc<WatcherManager>,
  InternedPath,
  Arc<Notify>,
  Arc<AtomicUsize>,
) {
  let temp = tempfile::tempdir().expect("create directory");
  let context = InternedPath::from(temp.path());
  let file = InternedPath::from(context.join("entry.js"));
  std::fs::write(&file, "entry").expect("write entry");
  let release = Arc::new(Notify::new());
  let calls = Arc::new(AtomicUsize::new(0));
  let ignored = FsWatcherIgnored::Function(Arc::new({
    let release = Arc::clone(&release);
    let calls = Arc::clone(&calls);
    move |path| {
      let is_file = std::path::Path::new(&path) == file.as_ref() as &std::path::Path;
      let pause = is_file && calls.fetch_add(1, Ordering::SeqCst) < pauses;
      let release = Arc::clone(&release);
      Box::pin(async move {
        if pause {
          release.notified().await;
        }
        false
      })
    }
  }));
  let manager = Arc::new(WatcherManager::new(ignored, false));
  manager
    .update(
      Default::default(),
      (vec![context.clone()], vec![]),
      Default::default(),
    )
    .await
    .expect("register");
  (temp, manager, context, release, calls)
}

#[tokio::test]
async fn requests_during_a_scan_share_one_followup_scan() {
  let (_temp, manager, context, release, calls) = fixture().await;
  let start = SystemTime::UNIX_EPOCH;
  let first = manager.scan_contexts(start);
  tokio::pin!(first);
  assert!(futures::poll!(&mut first).is_pending());
  let second = manager.scan_contexts(start);
  let third = manager.scan_contexts(start);
  tokio::pin!(second, third);
  assert!(futures::poll!(&mut second).is_pending());
  assert!(futures::poll!(&mut third).is_pending());
  let late = InternedPath::from(context.join("late.js"));
  std::fs::write(&late, "late").expect("write during scan");
  release.notify_one();
  let (_, second, third) = timeout(Duration::from_secs(5), async {
    tokio::join!(first, second, third)
  })
  .await
  .expect("scans finish");
  assert!(
    second.contains(&late),
    "followup must observe files added during the first scan"
  );
  assert!(
    third.contains(&late),
    "every waiter receives the followup result"
  );
  assert_eq!(
    calls.load(Ordering::SeqCst),
    2,
    "one active scan plus one shared followup"
  );
}

#[tokio::test]
async fn queued_scan_does_not_enter_a_replacement_watcher() {
  let (_temp, manager, context, release, calls) = fixture().await;
  let first = manager.scan_contexts(SystemTime::UNIX_EPOCH);
  let queued = manager.scan_contexts(SystemTime::UNIX_EPOCH);
  tokio::pin!(first, queued);
  assert!(futures::poll!(&mut first).is_pending());
  assert!(futures::poll!(&mut queued).is_pending());
  manager.reset();
  manager
    .update(
      Default::default(),
      (vec![], vec![context.clone()]),
      Default::default(),
    )
    .await
    .expect("unregister");
  manager.reset();
  manager
    .update(
      Default::default(),
      (vec![context.clone()], vec![]),
      Default::default(),
    )
    .await
    .expect("register replacement");
  release.notify_one();
  timeout(Duration::from_secs(5), async {
    tokio::join!(first, queued)
  })
  .await
  .expect("old scans finish");
  assert_eq!(
    calls.load(Ordering::SeqCst),
    1,
    "queued old scan must not read replacement"
  );
  let file = InternedPath::from(context.join("entry.js"));
  assert!(manager.entry(&file).is_none());
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  assert!(
    manager.entry(&file).is_some(),
    "replacement scans independently"
  );
}

#[tokio::test]
async fn coalescing_preserves_each_start_time() {
  let (_temp, manager, _context, release, calls) = fixture().await;
  let first = manager.scan_contexts(SystemTime::UNIX_EPOCH);
  let old = manager.scan_contexts(SystemTime::UNIX_EPOCH);
  let future = manager.scan_contexts(SystemTime::now() + Duration::from_secs(60));
  tokio::pin!(first, old, future);
  assert!(futures::poll!(&mut first).is_pending());
  assert!(futures::poll!(&mut old).is_pending());
  assert!(futures::poll!(&mut future).is_pending());
  release.notify_one();
  let (_, old, future) = timeout(Duration::from_secs(5), async {
    tokio::join!(first, old, future)
  })
  .await
  .expect("scans finish");
  assert!(!old.is_empty());
  assert!(
    future.is_empty(),
    "a future subscription must not reuse older notifications"
  );
  assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn cancelled_scan_does_not_finish_initialization_or_block_a_retry() {
  let (_temp, manager, context, _release, calls) = fixture().await;
  {
    let first = manager.scan_contexts(SystemTime::UNIX_EPOCH);
    tokio::pin!(first);
    assert!(futures::poll!(&mut first).is_pending());
    assert!(
      manager
        .watchers
        .get(&context)
        .expect("watcher")
        .initial_scan_finished
        .is_none()
    );
  }
  timeout(
    Duration::from_secs(5),
    manager.scan_contexts(SystemTime::UNIX_EPOCH),
  )
  .await
  .expect("retry finishes");
  assert!(
    manager
      .watchers
      .get(&context)
      .expect("watcher")
      .initial_scan_finished
      .is_some()
  );
  assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn absent_directory_finishes_initial_scan_with_symlink_following() {
  let temp = tempfile::tempdir().expect("create directory");
  let context = InternedPath::from(temp.path().join("absent"));
  let manager = WatcherManager::new(FsWatcherIgnored::default(), true);
  manager
    .update(
      Default::default(),
      (vec![context.clone()], vec![]),
      Default::default(),
    )
    .await
    .expect("register absent context");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  assert!(
    manager
      .watchers
      .get(&context)
      .expect("watcher")
      .initial_scan_finished
      .is_some(),
    "missing directory still completes its initial scan"
  );
}

#[tokio::test]
async fn an_earlier_subscription_can_share_a_later_subscriptions_scan() {
  let (_temp, manager, context, release, calls) = fixture().await;
  let first = manager.scan_contexts(SystemTime::UNIX_EPOCH);
  let future = manager.scan_contexts(SystemTime::now() + Duration::from_secs(60));
  let old = manager.scan_contexts(SystemTime::UNIX_EPOCH);
  tokio::pin!(first, future, old);
  assert!(futures::poll!(&mut first).is_pending());
  assert!(futures::poll!(&mut future).is_pending());
  assert!(futures::poll!(&mut old).is_pending());
  let directory = InternedPath::from(context.join("new-directory"));
  std::fs::create_dir(&directory).expect("create directory during scan");
  release.notify_one();
  let (_, future, old) = timeout(Duration::from_secs(5), async {
    tokio::join!(first, future, old)
  })
  .await
  .expect("scans finish");
  assert!(future.is_empty());
  assert!(old.contains(&InternedPath::from(context.join("entry.js"))));
  assert!(old.contains(&directory));
  assert_eq!(calls.load(Ordering::SeqCst), 2);
}

async fn mixed_scan_requests(noninitial_first: bool) {
  let (_temp, manager, context, release, calls) = fixture().await;
  let first = manager.scan_contexts(SystemTime::UNIX_EPOCH);
  tokio::pin!(first);
  assert!(futures::poll!(&mut first).is_pending());
  let start = SystemTime::now();
  let initial = manager.scan_contexts(start);
  let recovered = manager.scan(
    vec![context.clone()],
    crate::dirwatch::ScanReason::Recovered,
  );
  tokio::pin!(initial, recovered);
  if noninitial_first {
    assert!(futures::poll!(&mut recovered).is_pending());
    assert!(futures::poll!(&mut initial).is_pending());
  } else {
    assert!(futures::poll!(&mut initial).is_pending());
    assert!(futures::poll!(&mut recovered).is_pending());
  }
  let late = InternedPath::from(context.join("old-mtime.js"));
  let file = std::fs::File::create(&late).expect("create file after readdir");
  file
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1))
    .expect("set old mtime");
  release.notify_one();
  let (_, initial, recovered) = timeout(Duration::from_secs(5), async {
    tokio::join!(first, initial, recovered)
  })
  .await
  .expect("scans finish");
  let time = manager
    .entry(&late)
    .expect("file recorded")
    .time
    .expect("time recorded");
  assert_eq!(
    time.accuracy(),
    0,
    "noninitial scan must win regardless of queue order"
  );
  assert!(time.safe_time() >= rspack_util::time::system_time_to_millis(start));
  assert!(
    initial.contains(&late),
    "initial subscriber receives the observed change even with old mtime"
  );
  assert!(
    recovered == vec![late],
    "recovery reports the new file while the unchanged file stays silent"
  );
  assert_eq!(
    calls.load(Ordering::SeqCst),
    2,
    "mixed requests share one followup"
  );
}

#[tokio::test]
async fn initial_request_then_recovery_share_an_observed_scan() {
  mixed_scan_requests(false).await;
}

#[tokio::test]
async fn recovery_then_initial_request_share_an_observed_scan() {
  mixed_scan_requests(true).await;
}

#[tokio::test]
async fn a_cancelled_recovery_request_does_not_change_later_initial_scans() {
  let (_temp, manager, context, release, _calls) = fixture().await;
  let first = manager.scan_contexts(SystemTime::UNIX_EPOCH);
  tokio::pin!(first);
  assert!(futures::poll!(&mut first).is_pending());
  {
    let recovered = manager.scan(
      vec![context.clone()],
      crate::dirwatch::ScanReason::Recovered,
    );
    tokio::pin!(recovered);
    assert!(futures::poll!(&mut recovered).is_pending());
  }
  release.notify_one();
  timeout(Duration::from_secs(5), first)
    .await
    .expect("active scan finishes");
  let late = InternedPath::from(context.join("later.js"));
  let file = std::fs::File::create(&late).expect("create later file");
  file
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1))
    .expect("set old mtime");
  manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  let time = manager.entry(&late).expect("entry").time.expect("time");
  assert!(
    time.accuracy() > 0,
    "cancelled request must not promote an unrelated scan"
  );
}

#[tokio::test]
async fn cancellation_of_a_promoted_scan_preserves_the_pending_observed_mode() {
  let (_temp, manager, context, release, calls) = fixture_pausing(2).await;
  let first = manager.scan_contexts(SystemTime::UNIX_EPOCH);
  tokio::pin!(first);
  assert!(futures::poll!(&mut first).is_pending());
  let mut leader = Box::pin(manager.scan_contexts(SystemTime::UNIX_EPOCH));
  let mut survivor = Box::pin(manager.scan_contexts(SystemTime::UNIX_EPOCH));
  assert!(futures::poll!(&mut leader).is_pending());
  assert!(futures::poll!(&mut survivor).is_pending());
  {
    let recovery = manager.scan(
      vec![context.clone()],
      crate::dirwatch::ScanReason::Recovered,
    );
    tokio::pin!(recovery);
    assert!(futures::poll!(&mut recovery).is_pending());
  }
  release.notify_one();
  timeout(Duration::from_secs(5), first)
    .await
    .expect("first finishes");
  assert!(futures::poll!(&mut leader).is_pending());
  assert_eq!(
    calls.load(Ordering::SeqCst),
    2,
    "followup reached its predicate"
  );
  drop(leader);
  let late = InternedPath::from(context.join("after-cancellation.js"));
  let file = std::fs::File::create(&late).expect("create late file");
  file
    .set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(1))
    .expect("set old mtime");
  let result = timeout(Duration::from_secs(5), survivor)
    .await
    .expect("survivor finishes");
  assert!(result.contains(&late));
  assert_eq!(
    manager
      .entry(&late)
      .expect("entry")
      .time
      .expect("time")
      .accuracy(),
    0
  );
}

#[tokio::test]
async fn a_scan_event_after_removal_and_recreation_restores_time_info() {
  let temp = tempfile::tempdir().expect("create directory");
  let context = InternedPath::from(temp.path());
  let path = InternedPath::from(context.join("entry.js"));
  let mtime = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
  let file = std::fs::File::create(&path).expect("create file");
  file.set_modified(mtime).expect("set mtime");
  let manager = WatcherManager::new(FsWatcherIgnored::default(), false);
  manager
    .update(
      Default::default(),
      (vec![context.clone()], vec![]),
      Default::default(),
    )
    .await
    .expect("register context");
  let scanned = manager.scan_contexts(SystemTime::UNIX_EPOCH).await;
  assert!(scanned.contains(&path));
  std::fs::remove_file(&path).expect("remove test file");
  manager
    .process(
      &path,
      crate::FsEventKind::Remove,
      crate::state::Origin::Disk { appeared: false },
    )
    .await;
  assert!(manager.entry(&path).is_none());
  let file = std::fs::File::create(&path).expect("recreate file");
  file.set_modified(mtime).expect("restore same mtime");
  let start = crate::state::now();
  let events = manager
    .process(
      &path,
      crate::FsEventKind::Change,
      crate::state::Origin::Scan,
    )
    .await
    .expect("scan event reports current file");
  assert!(events.contains(&(context, crate::FsEventKind::Change)));
  let time = manager
    .entry(&path)
    .expect("file entry")
    .time
    .expect("recreated file has time info");
  assert_eq!(time.mtime(), mtime);
  assert_eq!(time.accuracy(), 0);
  assert!(time.safe_time() >= start);
}

#[tokio::test]
async fn a_resuming_scan_preserves_a_same_mtime_disk_observation() {
  let (_temp, manager, context, release, _calls) = fixture().await;
  let path = InternedPath::from(context.join("entry.js"));
  let mtime = SystemTime::now() + Duration::from_secs(60);
  std::fs::File::options()
    .write(true)
    .open(&path)
    .expect("open file")
    .set_modified(mtime)
    .expect("set future mtime");
  let scan = manager.scan_contexts(SystemTime::UNIX_EPOCH);
  tokio::pin!(scan);
  assert!(futures::poll!(&mut scan).is_pending());
  let events = manager
    .process(
      &path,
      crate::FsEventKind::Change,
      crate::state::Origin::Disk { appeared: false },
    )
    .await
    .expect("disk event");
  assert!(events.contains(&(context, crate::FsEventKind::Change)));
  let observed = manager
    .entry(&path)
    .expect("observed entry")
    .time
    .expect("observed time");
  assert_eq!(observed.accuracy(), 0);
  release.notify_one();
  let changes = timeout(Duration::from_secs(5), scan)
    .await
    .expect("scan resumes");
  assert!(changes.contains(&path));
  assert_eq!(
    manager.entry(&path).expect("scanned entry").time,
    Some(observed)
  );
  manager
    .process(
      &path,
      crate::FsEventKind::Change,
      crate::state::Origin::Scan,
    )
    .await;
  assert_eq!(
    manager.entry(&path).expect("reported entry").time,
    Some(observed)
  );
}
