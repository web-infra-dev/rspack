use std::{collections::HashSet, fs, time::Duration};

use notify::{Event, EventKind, RecursiveMode, event::Flag};
use rspack_paths::InternedPath;
use rspack_watcher::{FsEventKind, testing::DirectoryAnalyzer};
use tempfile::TempDir;

fn empty() -> (Vec<InternedPath>, Vec<InternedPath>) {
  (vec![], vec![])
}

fn added(path: &InternedPath) -> (Vec<InternedPath>, Vec<InternedPath>) {
  (vec![path.clone()], vec![])
}

fn removed(path: &InternedPath) -> (Vec<InternedPath>, Vec<InternedPath>) {
  (vec![], vec![path.clone()])
}

fn assert_patterns(analyzer: &DirectoryAnalyzer, expected: Vec<(InternedPath, RecursiveMode)>) {
  let actual: HashSet<_> = analyzer.analyze().into_iter().collect();
  assert_eq!(actual, expected.into_iter().collect::<HashSet<_>>());
}

async fn next_events(analyzer: &mut DirectoryAnalyzer) -> Vec<(InternedPath, FsEventKind)> {
  tokio::time::timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .expect("the event processor should report the dependency change")
}

#[tokio::test]
async fn test_shared_pattern_survives_until_the_last_dependency_is_removed() {
  let temp = TempDir::new().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let a = InternedPath::from(base.join("a.js"));
  let b = InternedPath::from(base.join("b.js"));
  fs::write(&a, "a").expect("create a.js");
  fs::write(&b, "b").expect("create b.js");
  let analyzer = DirectoryAnalyzer::default();
  let expected = vec![(InternedPath::from(base), RecursiveMode::NonRecursive)];

  analyzer
    .update((vec![a.clone(), b.clone()], vec![]), empty(), empty())
    .await
    .expect("register files");
  assert_patterns(&analyzer, expected.clone());
  assert_patterns(&analyzer, expected.clone());

  analyzer
    .update(removed(&a), empty(), empty())
    .await
    .expect("remove a.js");
  assert_patterns(&analyzer, expected.clone());
  analyzer
    .update(empty(), empty(), empty())
    .await
    .expect("empty update");
  assert_patterns(&analyzer, expected);

  analyzer
    .update(removed(&b), empty(), empty())
    .await
    .expect("remove b.js");
  assert_patterns(&analyzer, vec![]);
  analyzer
    .update(empty(), empty(), empty())
    .await
    .expect("empty update");
  assert_patterns(&analyzer, vec![]);
}

#[tokio::test]
async fn test_path_stays_watched_while_another_subscription_set_references_it() {
  let temp = TempDir::new().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let path = InternedPath::from(base.join("a.js"));
  let analyzer = DirectoryAnalyzer::default();
  let expected = vec![(InternedPath::from(base), RecursiveMode::NonRecursive)];

  analyzer
    .update(added(&path), empty(), added(&path))
    .await
    .expect("register in two sets");
  assert_patterns(&analyzer, expected.clone());
  analyzer
    .update(removed(&path), empty(), empty())
    .await
    .expect("remove file subscription");
  assert_patterns(&analyzer, expected);
  analyzer
    .update(empty(), empty(), removed(&path))
    .await
    .expect("remove missing subscription");
  assert_patterns(&analyzer, vec![]);
}

#[tokio::test]
async fn test_invalidation_recomputes_the_existing_ancestor_and_watch_mode() {
  let temp = TempDir::new().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let parent = InternedPath::from(base.join("new"));
  let nested = InternedPath::from(base.join("new/nested"));
  let missing = InternedPath::from(base.join("new/nested/a.js"));
  let analyzer = DirectoryAnalyzer::default();
  let original = vec![(InternedPath::from(base), RecursiveMode::Recursive)];

  analyzer
    .update(empty(), empty(), added(&missing))
    .await
    .expect("register missing path");
  assert_patterns(&analyzer, original.clone());

  fs::create_dir_all(&nested).expect("create nearer ancestor");
  analyzer.invalidate_path(&parent);
  analyzer
    .update(empty(), empty(), empty())
    .await
    .expect("empty update");
  assert_patterns(&analyzer, vec![(nested, RecursiveMode::NonRecursive)]);

  fs::remove_dir_all(&parent).expect("remove nearer ancestor");
  analyzer.invalidate_path(&parent);
  analyzer
    .update(empty(), empty(), empty())
    .await
    .expect("empty update");
  assert_patterns(&analyzer, original);
}

#[tokio::test]
async fn test_missing_to_files_migration_refreshes_the_pattern() {
  let temp = TempDir::new().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let parent = InternedPath::from(base.join("new"));
  let path = InternedPath::from(base.join("new/a.js"));
  let analyzer = DirectoryAnalyzer::default();

  analyzer
    .update(empty(), empty(), added(&path))
    .await
    .expect("register missing path");
  assert_patterns(
    &analyzer,
    vec![(InternedPath::from(base), RecursiveMode::Recursive)],
  );
  fs::create_dir(&parent).expect("create parent");
  fs::write(&path, "hello").expect("create dependency");
  analyzer
    .update(added(&path), empty(), removed(&path))
    .await
    .expect("migrate to files");
  assert_patterns(&analyzer, vec![(parent, RecursiveMode::NonRecursive)]);
}

#[tokio::test]
async fn test_missing_to_context_migration_refreshes_the_pattern() {
  let temp = TempDir::new().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let context = InternedPath::from(base.join("ctx"));
  let analyzer = DirectoryAnalyzer::default();

  analyzer
    .update(empty(), empty(), added(&context))
    .await
    .expect("register missing context");
  assert_patterns(
    &analyzer,
    vec![(InternedPath::from(base), RecursiveMode::NonRecursive)],
  );
  fs::create_dir(&context).expect("create context");
  analyzer
    .update(empty(), added(&context), removed(&context))
    .await
    .expect("migrate to contexts");
  assert_patterns(&analyzer, vec![(context, RecursiveMode::NonRecursive)]);
}

#[tokio::test]
async fn test_inferred_context_removal_refreshes_the_pattern() {
  let temp = TempDir::new().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let context = InternedPath::from(base.join("ctx"));
  let child = InternedPath::from(base.join("ctx/a.js"));
  fs::create_dir(&context).expect("create context");
  fs::write(&child, "hello").expect("create child");
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(empty(), added(&context), empty())
    .await
    .expect("register context");
  assert_patterns(
    &analyzer,
    vec![(context.clone(), RecursiveMode::NonRecursive)],
  );

  fs::remove_dir_all(&context).expect("remove context");
  analyzer.on_native_event(
    Event::new(EventKind::Remove(notify::event::RemoveKind::File))
      .add_path(child.as_ref().to_path_buf()),
  );
  assert!(
    next_events(&mut analyzer)
      .await
      .contains(&(context, FsEventKind::Remove))
  );
  analyzer
    .update(empty(), empty(), empty())
    .await
    .expect("empty update");
  assert_patterns(
    &analyzer,
    vec![(InternedPath::from(base), RecursiveMode::NonRecursive)],
  );
}

async fn assert_scan_appearance_refreshes_context(background_scan: bool) {
  let temp = TempDir::new().expect("create fixture");
  let base = temp.path().canonicalize().expect("resolve fixture");
  let context = InternedPath::from(base.join("ctx"));
  let child = InternedPath::from(base.join("ctx/a.js"));
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(empty(), added(&context), empty())
    .await
    .expect("register absent context");
  assert_patterns(
    &analyzer,
    vec![(InternedPath::from(base), RecursiveMode::NonRecursive)],
  );
  fs::create_dir(&context).expect("create context after analysis");
  fs::write(&child, "hello").expect("create child");

  if background_scan {
    analyzer.scan(std::time::UNIX_EPOCH);
  } else {
    analyzer.on_scanned_change(&child);
  }
  assert!(
    next_events(&mut analyzer)
      .await
      .contains(&(context.clone(), FsEventKind::Change))
  );
  analyzer
    .update(empty(), empty(), empty())
    .await
    .expect("empty update");
  assert_patterns(&analyzer, vec![(context, RecursiveMode::NonRecursive)]);
}

#[tokio::test]
async fn test_context_scan_refreshes_the_parent_context() {
  assert_scan_appearance_refreshes_context(false).await;
}

#[tokio::test]
async fn test_background_scan_refreshes_the_context() {
  assert_scan_appearance_refreshes_context(true).await;
}

#[tokio::test]
async fn test_rescan_invalidates_before_empty_paths_and_unsupported_kind_returns() {
  for include_path in [false, true] {
    let temp = TempDir::new().expect("create fixture");
    let base = temp.path().canonicalize().expect("resolve fixture");
    let context = InternedPath::from(base.join("ctx"));
    fs::create_dir(&context).expect("create context");
    let analyzer = DirectoryAnalyzer::default();
    analyzer
      .update(empty(), added(&context), empty())
      .await
      .expect("register context");
    assert_patterns(
      &analyzer,
      vec![(context.clone(), RecursiveMode::NonRecursive)],
    );
    fs::remove_dir(&context).expect("remove context without a native removal event");

    let mut event = Event::new(EventKind::Other).set_flag(Flag::Rescan);
    if include_path {
      event = event.add_path(context.as_ref().to_path_buf());
    }
    analyzer.on_native_event(event);
    analyzer
      .update(empty(), empty(), empty())
      .await
      .expect("empty update");
    assert_patterns(
      &analyzer,
      vec![(InternedPath::from(base), RecursiveMode::NonRecursive)],
    );
  }
}

#[cfg(unix)]
mod symlinks {
  use std::path::PathBuf;

  use super::*;

  fn external_link() -> (TempDir, PathBuf, PathBuf) {
    let temp = TempDir::new().expect("create fixture");
    let base = temp.path().canonicalize().expect("resolve fixture");
    let project = base.join("project");
    let target = base.join("external");
    fs::create_dir(&project).expect("create project");
    std::os::unix::fs::symlink(&target, project.join("link")).expect("create external symlink");
    (temp, project, target)
  }

  #[tokio::test]
  async fn test_dangling_symlink_recovery_refreshes_an_unchanged_missing_dependency() {
    let (_temp, project, target) = external_link();
    let missing = InternedPath::from(project.join("link/a.js"));
    let analyzer = DirectoryAnalyzer::default();
    analyzer
      .update(empty(), empty(), added(&missing))
      .await
      .expect("register missing dependency");
    assert_patterns(
      &analyzer,
      vec![(
        InternedPath::from(project.clone()),
        RecursiveMode::Recursive,
      )],
    );

    fs::create_dir(&target).expect("create external target");
    assert!(!missing.exists());
    analyzer
      .update(empty(), empty(), empty())
      .await
      .expect("unchanged missing subscription");
    assert_patterns(
      &analyzer,
      vec![(
        InternedPath::from(project.join("link")),
        RecursiveMode::NonRecursive,
      )],
    );
  }

  #[tokio::test]
  async fn test_file_target_becoming_a_directory_refreshes_a_descendant_dependency() {
    let (_temp, project, target) = external_link();
    fs::write(&target, "regular file").expect("create external file target");
    let missing = InternedPath::from(project.join("link/a.js"));
    let analyzer = DirectoryAnalyzer::default();
    analyzer
      .update(empty(), empty(), added(&missing))
      .await
      .expect("register missing dependency");
    assert_patterns(
      &analyzer,
      vec![(
        InternedPath::from(project.clone()),
        RecursiveMode::Recursive,
      )],
    );

    fs::remove_file(&target).expect("remove external file");
    fs::create_dir(&target).expect("replace external file with directory");
    analyzer
      .update(empty(), empty(), empty())
      .await
      .expect("empty update");
    assert_patterns(
      &analyzer,
      vec![(
        InternedPath::from(project.join("link")),
        RecursiveMode::NonRecursive,
      )],
    );
  }

  #[tokio::test]
  async fn test_file_target_becoming_a_directory_refreshes_context_and_missing_roots() {
    for is_context in [false, true] {
      let (_temp, project, target) = external_link();
      fs::write(&target, "regular file").expect("create external file target");
      let link = InternedPath::from(project.join("link"));
      let analyzer = DirectoryAnalyzer::default();
      if is_context {
        analyzer
          .update(empty(), added(&link), empty())
          .await
          .expect("register context symlink");
      } else {
        analyzer
          .update(empty(), empty(), added(&link))
          .await
          .expect("register missing symlink");
      }
      assert_patterns(
        &analyzer,
        vec![(InternedPath::from(project), RecursiveMode::NonRecursive)],
      );

      fs::remove_file(&target).expect("remove external file");
      fs::create_dir(&target).expect("replace external file with directory");
      analyzer
        .update(empty(), empty(), empty())
        .await
        .expect("empty update");
      assert_patterns(&analyzer, vec![(link, RecursiveMode::NonRecursive)]);
    }
  }
}
