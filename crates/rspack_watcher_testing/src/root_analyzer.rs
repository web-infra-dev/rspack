use rspack_paths::InternedPath;

use crate::{
  analyzer::{Analyzer, root::WatcherRootAnalyzer},
  registry::Registry,
};

#[tokio::test]
async fn removing_one_subscription_kind_preserves_the_other() {
  let temp = tempfile::tempdir().expect("directory");
  let root = InternedPath::from(temp.path());
  let registry = Registry::default();
  let analyzer = WatcherRootAnalyzer::default();
  registry
    .update(
      (std::iter::empty(), std::iter::empty()),
      (vec![root.clone()].into_iter(), std::iter::empty()),
      (vec![root.clone()].into_iter(), std::iter::empty()),
    )
    .await
    .expect("register");
  assert_eq!(analyzer.analyze(registry.access())[0].path, root);
  assert_eq!(
    analyzer.analyze(registry.access()).len(),
    1,
    "repeated delta"
  );
  registry.reset();
  registry
    .update(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), vec![root.clone()].into_iter()),
    )
    .await
    .expect("remove missing");
  assert_eq!(analyzer.analyze(registry.access())[0].path, root);
}

#[tokio::test]
async fn a_context_keeps_coverage_outside_its_only_registered_child() {
  let temp = tempfile::tempdir().expect("directory");
  let root = InternedPath::from(temp.path());
  let nested = InternedPath::from(root.join("nested"));
  std::fs::create_dir(&nested).expect("nested");
  let registry = Registry::default();
  let analyzer = WatcherRootAnalyzer::default();
  registry
    .update(
      (
        vec![InternedPath::from(nested.join("entry.js"))].into_iter(),
        std::iter::empty(),
      ),
      (vec![root.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
    )
    .await
    .expect("register");
  assert_eq!(analyzer.analyze(registry.access())[0].path, root);
  assert_eq!(
    analyzer.analyze(registry.access()).len(),
    1,
    "repeated delta"
  );
  registry.reset();
  registry
    .update(
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), vec![root].into_iter()),
      (std::iter::empty(), std::iter::empty()),
    )
    .await
    .expect("remove context");
  assert_eq!(analyzer.analyze(registry.access())[0].path, nested);
}

#[tokio::test]
async fn removing_the_last_subscription_releases_the_root() {
  let temp = tempfile::tempdir().expect("directory");
  let file = InternedPath::from(temp.path().join("entry.js"));
  let registry = Registry::default();
  let analyzer = WatcherRootAnalyzer::default();
  registry
    .update(
      (vec![file.clone()].into_iter(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
      (std::iter::empty(), std::iter::empty()),
    )
    .await
    .expect("register");
  assert_eq!(analyzer.analyze(registry.access()).len(), 1);
  assert_eq!(
    analyzer.analyze(registry.access()).len(),
    1,
    "repeated delta"
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
