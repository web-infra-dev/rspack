use rspack_paths::InternedPath;

use crate::{
  WatchPattern,
  analyzer::{Analyzer, directories::WatcherDirectoriesAnalyzer},
  registry::Registry,
};

#[tokio::test]
async fn context_requires_one_recursive_pattern_for_its_directory() {
  let temp = tempfile::tempdir().expect("directory");
  let context = InternedPath::from(temp.path());
  let file = InternedPath::from(context.join("entry.js"));
  std::fs::write(&file, "entry").expect("file");
  let registry = Registry::default();
  registry
    .update(
      (vec![file].into_iter(), std::iter::empty()),
      (vec![context.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
    )
    .await
    .expect("register");
  let analyzer = WatcherDirectoriesAnalyzer::default();
  assert_eq!(
    analyzer.analyze(registry.access()),
    vec![WatchPattern {
      path: context,
      mode: notify::RecursiveMode::Recursive
    }]
  );
}

#[tokio::test]
async fn recursive_requirement_is_removed_with_the_last_context_subscription() {
  let temp = tempfile::tempdir().expect("directory");
  let context = InternedPath::from(temp.path());
  let file = InternedPath::from(context.join("entry.js"));
  std::fs::write(&file, "entry").expect("file");
  let registry = Registry::default();
  let analyzer = WatcherDirectoriesAnalyzer::default();
  registry
    .update(
      (vec![file.clone()].into_iter(), std::iter::empty()),
      (vec![context.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
    )
    .await
    .expect("register");
  let first = analyzer.analyze(registry.access());
  assert_eq!(
    analyzer.analyze(registry.access()),
    first,
    "repeated analysis must not double-count the delta"
  );
  registry.reset();
  registry
    .update(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), vec![context.clone()].into_iter()),
      (std::iter::empty(), std::iter::empty()),
    )
    .await
    .expect("remove context");
  assert_eq!(
    analyzer.analyze(registry.access()),
    vec![WatchPattern {
      path: context,
      mode: notify::RecursiveMode::NonRecursive
    }]
  );
  registry.reset();
  registry
    .update(
      (std::iter::empty(), vec![file].into_iter()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
    )
    .await
    .expect("remove file");
  assert!(analyzer.analyze(registry.access()).is_empty());
}

#[tokio::test]
async fn a_missing_parent_can_recover_without_a_subscription_delta() {
  let temp = tempfile::tempdir().expect("directory");
  let root = InternedPath::from(temp.path());
  let parent = InternedPath::from(root.join("parent"));
  let file = InternedPath::from(parent.join("file.js"));
  let registry = Registry::default();
  let analyzer = WatcherDirectoriesAnalyzer::default();
  registry
    .update(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (vec![file.clone()].into_iter(), std::iter::empty()),
    )
    .await
    .expect("register");
  assert_eq!(
    analyzer.analyze(registry.access()),
    vec![WatchPattern {
      path: root.clone(),
      mode: notify::RecursiveMode::NonRecursive
    }]
  );
  registry.reset();
  std::fs::create_dir(&parent).expect("restore parent");
  analyzer.invalidate_directory(&parent);
  assert_eq!(
    analyzer.analyze(registry.access()),
    vec![WatchPattern {
      path: parent.clone(),
      mode: notify::RecursiveMode::NonRecursive
    }]
  );
  registry
    .update(
      (vec![file.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), vec![file].into_iter()),
    )
    .await
    .expect("migrate missing to file");
  assert_eq!(
    analyzer.analyze(registry.access()),
    vec![WatchPattern {
      path: parent.clone(),
      mode: notify::RecursiveMode::NonRecursive
    }]
  );
  registry.reset();
  std::fs::remove_dir(&parent).expect("remove parent");
  analyzer.invalidate_directory(&parent);
  assert_eq!(
    analyzer.analyze(registry.access()),
    vec![WatchPattern {
      path: root,
      mode: notify::RecursiveMode::NonRecursive
    }]
  );
}

async fn benchmark_directory_analyzer_delta(scattered: bool) {
  for count in [1, 1_000, 10_000] {
    let temp = tempfile::tempdir().expect("directory");
    let registry = Registry::default();
    let analyzer = WatcherDirectoriesAnalyzer::default();
    let missing = (0..count).map(|i| {
      InternedPath::from(temp.path().join(if scattered {
        format!("missing-dir-{i}/entry.js")
      } else {
        format!("missing-{i}.js")
      }))
    });
    registry
      .update(
        (std::iter::empty(), std::iter::empty()),
        (std::iter::empty(), std::iter::empty()),
        (missing, std::iter::empty()),
      )
      .await
      .expect("register");
    analyzer.analyze(registry.access());
    registry.reset();
    let mut samples = Vec::new();
    for i in 0..120 {
      if i == 20 {
        eprintln!("WATCHER_FS_SAMPLE_BEGIN analyzer scattered={scattered} missing={count}");
      }
      let start = std::time::Instant::now();
      assert_eq!(analyzer.analyze(registry.access()).len(), 1);
      if i >= 20 {
        samples.push(start.elapsed().as_nanos());
      }
    }
    eprintln!("WATCHER_FS_SAMPLE_END");
    samples.sort_unstable();
    println!(
      "scattered={scattered}, missing={count}, n=100, p50_ns={}, p95_ns={}",
      samples[50], samples[95]
    );
  }
}

#[tokio::test]
#[ignore = "manual directory analyzer benchmark"]
async fn benchmark_empty_directory_analyzer_delta() {
  benchmark_directory_analyzer_delta(false).await;
}

#[tokio::test]
#[ignore = "manual directory analyzer benchmark"]
async fn benchmark_scattered_missing_directory_analyzer_delta() {
  benchmark_directory_analyzer_delta(true).await;
}

#[tokio::test]
async fn recursive_parent_covers_child_patterns() {
  let temp = tempfile::tempdir().expect("directory");
  let context = InternedPath::from(temp.path());
  let file = InternedPath::from(context.join("nested/entry.js"));
  std::fs::create_dir_all(file.parent().expect("parent")).expect("nested");
  std::fs::write(&file, "entry").expect("file");
  let registry = Registry::default();
  registry
    .update(
      (vec![file].into_iter(), std::iter::empty()),
      (vec![context.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
    )
    .await
    .expect("register");
  let analyzer = WatcherDirectoriesAnalyzer::default();
  assert_eq!(
    analyzer.analyze(registry.access()),
    vec![WatchPattern {
      path: context,
      mode: notify::RecursiveMode::Recursive
    }]
  );
}

#[tokio::test]
async fn an_empty_delta_keeps_cached_directories_until_a_directory_event() {
  let temp = tempfile::tempdir().expect("directory");
  let root = InternedPath::from(temp.path());
  let parent = InternedPath::from(root.join("parent"));
  let registry = Registry::default();
  registry
    .update(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (
        vec![InternedPath::from(parent.join("entry.js"))].into_iter(),
        std::iter::empty(),
      ),
    )
    .await
    .expect("register");
  let analyzer = WatcherDirectoriesAnalyzer::default();
  let before = analyzer.analyze(registry.access());
  registry.reset();
  std::fs::create_dir(&parent).expect("restore directory");
  assert_eq!(
    analyzer.analyze(registry.access()),
    before,
    "an empty delta must reuse the cached ancestor"
  );
}

#[tokio::test]
async fn directory_events_refresh_only_the_affected_cached_ancestors() {
  let temp = tempfile::tempdir().expect("directory");
  let root = InternedPath::from(temp.path());
  let a = InternedPath::from(root.join("a"));
  let b = InternedPath::from(root.join("b"));
  let registry = Registry::default();
  registry
    .update(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (
        vec![
          InternedPath::from(a.join("entry.js")),
          InternedPath::from(b.join("entry.js")),
        ]
        .into_iter(),
        std::iter::empty(),
      ),
    )
    .await
    .expect("register");
  let analyzer = WatcherDirectoriesAnalyzer::default();
  analyzer.analyze(registry.access());
  registry.reset();
  std::fs::create_dir(&a).expect("a");
  std::fs::create_dir(&b).expect("b");
  analyzer.invalidate_directory(&a);
  let patterns = analyzer.analyze(registry.access());
  assert_eq!(patterns.len(), 2);
  assert!(patterns.iter().any(|pattern| pattern.path == a));
  assert!(patterns.iter().any(|pattern| pattern.path == root));
  assert_eq!(analyzer.analyze(registry.access()), patterns);
  analyzer.invalidate_directory(&b);
  let patterns = analyzer.analyze(registry.access());
  assert_eq!(patterns.len(), 2);
  assert!(patterns.iter().any(|pattern| pattern.path == a));
  assert!(patterns.iter().any(|pattern| pattern.path == b));
}

async fn failed_watch_installation_can_retry(recovering: bool) {
  use std::sync::Arc;

  use tokio::sync::mpsc;

  use crate::{WatcherManager, disk_watcher::DiskWatcher, trigger::Trigger};

  for kind in ["file", "context", "missing"] {
    let temp = tempfile::tempdir().expect("directory");
    let root = InternedPath::from(temp.path().canonicalize().expect("canonicalize"));
    let parent = InternedPath::from(root.join("parent"));
    let unrelated = root.join("unrelated");
    if !recovering {
      std::fs::create_dir(&parent).expect("parent");
    }
    let path = if kind == "context" {
      parent.clone()
    } else {
      InternedPath::from(parent.join("entry.js"))
    };
    let state = Arc::new(WatcherManager::default());
    let mut files = vec![];
    let mut contexts = vec![];
    let mut missing = vec![InternedPath::from(unrelated.join("entry.js"))];
    match kind {
      "file" => files.push(path),
      "context" => contexts.push(path),
      _ => missing.push(path),
    }
    state
      .update((files, vec![]), (contexts, vec![]), (missing, vec![]))
      .await
      .expect("register");
    let analyzer = WatcherDirectoriesAnalyzer::default();
    let (tx, _rx) = mpsc::unbounded_channel();
    let trigger = Arc::new(Trigger::new(Arc::clone(&state), tx));
    let mut disk = DiskWatcher::new(false, None, trigger, Arc::clone(&state));
    let mut patterns = analyzer.analyze(state.registry.access());
    if recovering {
      disk
        .watch(patterns.into_iter(), |path| {
          analyzer.invalidate_directory(path)
        })
        .expect("ancestor watch");
      std::fs::create_dir(&parent).expect("restore parent");
      analyzer.invalidate_directory(&parent);
      patterns = analyzer.analyze(state.registry.access());
    }
    assert!(patterns.iter().any(|pattern| pattern.path == parent));
    std::fs::remove_dir(&parent).expect("remove before installation");
    let result = disk.watch(patterns.into_iter(), |path| {
      analyzer.invalidate_directory(path);
    });
    assert!(
      result.is_err(),
      "installation must fail for the removed directory"
    );
    state.reset();
    std::fs::create_dir(&unrelated).expect("unrelated directory");
    let patterns = analyzer.analyze(state.registry.access());
    assert_eq!(
      patterns,
      vec![WatchPattern {
        path: root.clone(),
        mode: notify::RecursiveMode::NonRecursive
      }],
      "retry must resolve the failed root without refreshing unrelated subscriptions, kind={kind}, recovering={recovering}"
    );
    disk
      .watch(patterns.into_iter(), |path| {
        analyzer.invalidate_directory(path)
      })
      .expect("retry watches existing ancestor");
    disk.close();
  }
}

#[tokio::test]
async fn a_failed_subscription_watch_invalidates_its_cached_directory() {
  failed_watch_installation_can_retry(false).await;
}

#[tokio::test]
async fn a_failed_recovery_watch_invalidates_its_cached_directory() {
  failed_watch_installation_can_retry(true).await;
}

#[cfg(target_os = "linux")]
fn owner_for_installation_retry(
  state: std::sync::Arc<crate::WatcherManager>,
) -> crate::FsWatcherInner {
  use std::sync::{Arc, Mutex, atomic::AtomicBool};

  let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
  let trigger = Arc::new(crate::trigger::Trigger::new(Arc::clone(&state), tx));
  crate::FsWatcherInner {
    disk_watcher: crate::disk_watcher::DiskWatcher::new(
      false,
      None,
      Arc::clone(&trigger),
      Arc::clone(&state),
    ),
    state,
    executor: crate::executor::Executor::new(rx, Some(0), Arc::new(AtomicBool::new(false))),
    backfill_sender: None,
    analyzer: Default::default(),
    polling: None,
    poll_interval: None,
    trigger: Arc::new(Mutex::new(Some(trigger))),
  }
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn owner_subscription_installation_failure_retries_without_a_delta() {
  use std::{sync::Arc, time::SystemTime};

  let temp = tempfile::tempdir().expect("directory");
  let root = InternedPath::from(temp.path().canonicalize().expect("canonicalize"));
  let parent = root.join("parent");
  std::fs::create_dir(&parent).expect("parent");
  let state = Arc::new(crate::WatcherManager::default());
  state
    .update(
      (vec![InternedPath::from(parent.join("entry.js"))], vec![]),
      Default::default(),
      Default::default(),
    )
    .await
    .expect("register");
  let mut owner = owner_for_installation_retry(state);
  owner.analyzer.analyze(owner.state.registry.access());
  owner.state.reset();
  std::fs::remove_dir(&parent).expect("remove before installation");
  assert!(
    owner
      .wait_for_event(
        Default::default(),
        Default::default(),
        Default::default(),
        SystemTime::now()
      )
      .await
      .is_err()
  );
  owner
    .wait_for_event(
      Default::default(),
      Default::default(),
      Default::default(),
      SystemTime::now(),
    )
    .await
    .expect("ordinary watch retry must attach to the ancestor");
  assert_eq!(
    owner.analyzer.analyze(owner.state.registry.access()),
    vec![WatchPattern {
      path: root,
      mode: notify::RecursiveMode::NonRecursive
    }]
  );
  owner.close().await.expect("close");
}

#[cfg(target_os = "linux")]
#[tokio::test]
async fn owner_recovery_installation_failure_retries_without_a_delta() {
  use std::sync::Arc;

  let temp = tempfile::tempdir().expect("directory");
  let root = InternedPath::from(temp.path().canonicalize().expect("canonicalize"));
  let parent = root.join("parent");
  let recovering = InternedPath::from(root.join("recovering"));
  std::fs::create_dir(&parent).expect("parent");
  std::fs::create_dir(&recovering).expect("recovering directory");
  let state = Arc::new(crate::WatcherManager::default());
  state
    .update(
      (
        vec![
          InternedPath::from(parent.join("entry.js")),
          InternedPath::from(recovering.join("entry.js")),
        ],
        vec![],
      ),
      Default::default(),
      Default::default(),
    )
    .await
    .expect("register");
  let mut owner = owner_for_installation_retry(state);
  owner.analyzer.analyze(owner.state.registry.access());
  owner.state.reset();
  std::fs::remove_dir(&parent).expect("remove before installation");
  let event = || crate::disk_watcher::RootEvent {
    path: recovering.clone(),
    kind: crate::FsEventKind::Create,
    appeared: true,
  };
  assert!(owner.refresh_root_watch(event()).is_err());
  owner
    .refresh_root_watch(event())
    .expect("directory recovery retry must attach to the ancestor");
  let patterns = owner.analyzer.analyze(owner.state.registry.access());
  assert_eq!(patterns.len(), 2);
  assert!(patterns.iter().any(|pattern| pattern.path == root));
  assert!(patterns.iter().any(|pattern| pattern.path == recovering));
  owner.close().await.expect("close");
}
