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
  analyzer.on_directory_event(&parent);
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
  analyzer.on_directory_event(&parent);
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
  analyzer.on_directory_event(&a);
  let patterns = analyzer.analyze(registry.access());
  assert_eq!(patterns.len(), 2);
  assert!(patterns.iter().any(|pattern| pattern.path == a));
  assert!(patterns.iter().any(|pattern| pattern.path == root));
  assert_eq!(analyzer.analyze(registry.access()), patterns);
  analyzer.on_directory_event(&b);
  let patterns = analyzer.analyze(registry.access());
  assert_eq!(patterns.len(), 2);
  assert!(patterns.iter().any(|pattern| pattern.path == a));
  assert!(patterns.iter().any(|pattern| pattern.path == b));
}
