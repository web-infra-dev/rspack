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
  assert_eq!(
    analyzer.analyze(registry.access()),
    vec![WatchPattern {
      path: root,
      mode: notify::RecursiveMode::NonRecursive
    }]
  );
}

#[tokio::test]
#[ignore = "manual directory analyzer benchmark"]
async fn benchmark_empty_directory_analyzer_delta() {
  for count in [1, 1_000, 10_000] {
    let temp = tempfile::tempdir().expect("directory");
    let registry = Registry::default();
    let analyzer = WatcherDirectoriesAnalyzer::default();
    let missing =
      (0..count).map(|i| InternedPath::from(temp.path().join(format!("missing-{i}.js"))));
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
      let start = std::time::Instant::now();
      assert_eq!(analyzer.analyze(registry.access()).len(), 1);
      if i >= 20 {
        samples.push(start.elapsed().as_micros());
      }
    }
    samples.sort_unstable();
    println!(
      "missing={count}, n=100, p50_us={}, p95_us={}",
      samples[50], samples[95]
    );
  }
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
