use std::{collections::HashSet, fs, time::Duration};

use notify::{Event, EventKind, RecursiveMode, event::Flag};
use rspack_fs::AbsPath;
use rspack_watcher::{FsEventKind, testing::DirectoryAnalyzer};
use tempfile::TempDir;

fn empty() -> (Vec<AbsPath>, Vec<AbsPath>) {
  (vec![], vec![])
}

fn added(path: &AbsPath) -> (Vec<AbsPath>, Vec<AbsPath>) {
  (vec![path.clone()], vec![])
}

fn removed(path: &AbsPath) -> (Vec<AbsPath>, Vec<AbsPath>) {
  (vec![], vec![path.clone()])
}

fn assert_patterns(analyzer: &DirectoryAnalyzer, expected: Vec<(AbsPath, RecursiveMode)>) {
  let actual: HashSet<_> = analyzer.analyze().into_iter().collect();
  assert_eq!(actual, expected.into_iter().collect::<HashSet<_>>());
}

async fn next_events(analyzer: &mut DirectoryAnalyzer) -> Vec<(AbsPath, FsEventKind)> {
  tokio::time::timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .expect("the event processor should report the dependency change")
}

#[tokio::test]
async fn test_shared_pattern_survives_until_the_last_dependency_is_removed() {
  let temp = TempDir::new().expect("create fixture");
  let base = AbsPath::try_from_std_path(&temp.path().canonicalize().expect("resolve fixture"))
    .expect("valid utf-8 fixture path");
  let a = base.join("a.js").expect("valid path");
  let b = base.join("b.js").expect("valid path");
  fs::write(&a, "a").expect("create a.js");
  fs::write(&b, "b").expect("create b.js");
  let analyzer = DirectoryAnalyzer::default();
  let expected = vec![(base, RecursiveMode::NonRecursive)];

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
  let base = AbsPath::try_from_std_path(&temp.path().canonicalize().expect("resolve fixture"))
    .expect("valid utf-8 fixture path");
  let path = base.join("a.js").expect("valid path");
  let analyzer = DirectoryAnalyzer::default();
  let expected = vec![(base, RecursiveMode::NonRecursive)];

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
  let base = AbsPath::try_from_std_path(&temp.path().canonicalize().expect("resolve fixture"))
    .expect("valid utf-8 fixture path");
  let parent = base.join("new").expect("valid path");
  let nested = base.join("new/nested").expect("valid path");
  let missing = base.join("new/nested/a.js").expect("valid path");
  let analyzer = DirectoryAnalyzer::default();
  let original = vec![(base, RecursiveMode::Recursive)];

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
  let base = AbsPath::try_from_std_path(&temp.path().canonicalize().expect("resolve fixture"))
    .expect("valid utf-8 fixture path");
  let parent = base.join("new").expect("valid path");
  let path = base.join("new/a.js").expect("valid path");
  let analyzer = DirectoryAnalyzer::default();

  analyzer
    .update(empty(), empty(), added(&path))
    .await
    .expect("register missing path");
  assert_patterns(&analyzer, vec![(base, RecursiveMode::Recursive)]);
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
  let base = AbsPath::try_from_std_path(&temp.path().canonicalize().expect("resolve fixture"))
    .expect("valid utf-8 fixture path");
  let context = base.join("ctx").expect("valid path");
  let analyzer = DirectoryAnalyzer::default();

  analyzer
    .update(empty(), empty(), added(&context))
    .await
    .expect("register missing context");
  assert_patterns(&analyzer, vec![(base, RecursiveMode::NonRecursive)]);
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
  let base = AbsPath::try_from_std_path(&temp.path().canonicalize().expect("resolve fixture"))
    .expect("valid utf-8 fixture path");
  let context = base.join("ctx").expect("valid path");
  let child = base.join("ctx/a.js").expect("valid path");
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
      .add_path(child.to_std_path_buf()),
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
  assert_patterns(&analyzer, vec![(base, RecursiveMode::NonRecursive)]);
}

async fn assert_scan_appearance_refreshes_context(background_scan: bool) {
  let temp = TempDir::new().expect("create fixture");
  let base = AbsPath::try_from_std_path(&temp.path().canonicalize().expect("resolve fixture"))
    .expect("valid utf-8 fixture path");
  let context = base.join("ctx").expect("valid path");
  let child = base.join("ctx/a.js").expect("valid path");
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(empty(), added(&context), empty())
    .await
    .expect("register absent context");
  assert_patterns(&analyzer, vec![(base, RecursiveMode::NonRecursive)]);
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
async fn test_scanner_remembers_a_removal_until_the_file_is_unregistered() {
  let temp = TempDir::new().expect("create fixture");
  let base = AbsPath::try_from_std_path(&temp.path().canonicalize().expect("resolve fixture"))
    .expect("valid utf-8 fixture path");
  let gone = base.join("gone.js").expect("valid path");
  let present = base.join("present.js").expect("valid path");
  fs::write(&present, "").expect("create present file");
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(added(&gone), empty(), empty())
    .await
    .expect("register absent file");
  analyzer.scan(std::time::SystemTime::UNIX_EPOCH);
  assert_eq!(
    next_events(&mut analyzer).await,
    vec![(gone.clone(), FsEventKind::Remove)]
  );

  analyzer
    .update(
      (vec![gone.clone(), present.clone()], vec![]),
      empty(),
      empty(),
    )
    .await
    .expect("rewatch both files");
  // The file scan sends removals before changes. A duplicate removal would
  // arrive before this present file's change.
  analyzer.scan(std::time::SystemTime::UNIX_EPOCH);
  assert_eq!(
    next_events(&mut analyzer).await,
    vec![(present, FsEventKind::Change)]
  );

  analyzer
    .update(removed(&gone), empty(), empty())
    .await
    .expect("unregister absent file");
  analyzer
    .update(added(&gone), empty(), empty())
    .await
    .expect("register absent file again");
  analyzer.scan(std::time::SystemTime::UNIX_EPOCH);
  assert_eq!(
    next_events(&mut analyzer).await,
    vec![(gone, FsEventKind::Remove)]
  );
}

#[tokio::test]
async fn test_rescan_invalidates_before_empty_paths_and_unsupported_kind_returns() {
  for include_path in [false, true] {
    let temp = TempDir::new().expect("create fixture");
    let base = AbsPath::try_from_std_path(&temp.path().canonicalize().expect("resolve fixture"))
      .expect("valid utf-8 fixture path");
    let context = base.join("ctx").expect("valid path");
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
      event = event.add_path(context.to_std_path_buf());
    }
    analyzer.on_native_event(event);
    analyzer
      .update(empty(), empty(), empty())
      .await
      .expect("empty update");
    assert_patterns(&analyzer, vec![(base, RecursiveMode::NonRecursive)]);
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
  async fn test_resolved_directory_symlink_retarget_refreshes_a_missing_dependency() {
    for has_subdirectory in [true, false] {
      let (_temp, project, target) = external_link();
      fs::create_dir(&target).expect("create initial target");
      if has_subdirectory {
        fs::create_dir(target.join("sub")).expect("create initial subdirectory");
      }
      let replacement = target.with_file_name("replacement");
      fs::create_dir(&replacement).expect("create replacement target");
      if !has_subdirectory {
        fs::create_dir(replacement.join("sub")).expect("create replacement subdirectory");
      }
      let missing = AbsPath::try_from_std_path(&project.join("link/sub/a.js")).expect("valid path");
      let analyzer = DirectoryAnalyzer::default();
      analyzer
        .update(empty(), empty(), added(&missing))
        .await
        .expect("register missing dependency");
      assert_patterns(
        &analyzer,
        vec![(
          AbsPath::try_from_std_path(&project.join(if has_subdirectory {
            "link/sub"
          } else {
            "link"
          }))
          .expect("valid path"),
          if has_subdirectory {
            RecursiveMode::NonRecursive
          } else {
            RecursiveMode::Recursive
          },
        )],
      );

      fs::remove_file(project.join("link")).expect("remove initial link");
      std::os::unix::fs::symlink(&replacement, project.join("link")).expect("retarget link");
      analyzer
        .update(empty(), empty(), empty())
        .await
        .expect("empty update after retargeting");
      assert_patterns(
        &analyzer,
        vec![(
          AbsPath::try_from_std_path(&project.join(if has_subdirectory {
            "link"
          } else {
            "link/sub"
          }))
          .expect("valid path"),
          if has_subdirectory {
            RecursiveMode::Recursive
          } else {
            RecursiveMode::NonRecursive
          },
        )],
      );
    }
  }

  #[tokio::test]
  async fn test_resolved_directory_symlink_unlink_refreshes_a_missing_dependency() {
    for has_subdirectory in [true, false] {
      let (_temp, project, target) = external_link();
      fs::create_dir(&target).expect("create target directory");
      if has_subdirectory {
        fs::create_dir(target.join("sub")).expect("create target subdirectory");
      }
      let missing = AbsPath::try_from_std_path(&project.join("link/sub/a.js")).expect("valid path");
      let analyzer = DirectoryAnalyzer::default();
      analyzer
        .update(empty(), empty(), added(&missing))
        .await
        .expect("register missing dependency");
      assert_patterns(
        &analyzer,
        vec![(
          AbsPath::try_from_std_path(&project.join(if has_subdirectory {
            "link/sub"
          } else {
            "link"
          }))
          .expect("valid path"),
          if has_subdirectory {
            RecursiveMode::NonRecursive
          } else {
            RecursiveMode::Recursive
          },
        )],
      );

      fs::remove_file(project.join("link")).expect("unlink resolved directory symlink");
      analyzer
        .update(empty(), empty(), empty())
        .await
        .expect("empty update after unlinking");
      assert_patterns(
        &analyzer,
        vec![(
          AbsPath::try_from_std_path(&project).expect("valid path"),
          RecursiveMode::Recursive,
        )],
      );
    }
  }

  #[tokio::test]
  async fn test_dangling_symlink_recovery_refreshes_an_unchanged_missing_dependency() {
    let (_temp, project, target) = external_link();
    let missing = AbsPath::try_from_std_path(&project.join("link/a.js")).expect("valid path");
    let analyzer = DirectoryAnalyzer::default();
    analyzer
      .update(empty(), empty(), added(&missing))
      .await
      .expect("register missing dependency");
    assert_patterns(
      &analyzer,
      vec![(
        AbsPath::try_from_std_path(&project).expect("valid path"),
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
        AbsPath::try_from_std_path(&project.join("link")).expect("valid path"),
        RecursiveMode::NonRecursive,
      )],
    );
  }

  #[tokio::test]
  async fn test_file_target_becoming_a_directory_refreshes_a_descendant_dependency() {
    let (_temp, project, target) = external_link();
    fs::write(&target, "regular file").expect("create external file target");
    let missing = AbsPath::try_from_std_path(&project.join("link/a.js")).expect("valid path");
    let analyzer = DirectoryAnalyzer::default();
    analyzer
      .update(empty(), empty(), added(&missing))
      .await
      .expect("register missing dependency");
    assert_patterns(
      &analyzer,
      vec![(
        AbsPath::try_from_std_path(&project).expect("valid path"),
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
        AbsPath::try_from_std_path(&project.join("link")).expect("valid path"),
        RecursiveMode::NonRecursive,
      )],
    );
  }

  #[tokio::test]
  async fn test_file_target_becoming_a_directory_refreshes_context_and_missing_roots() {
    for is_context in [false, true] {
      let (_temp, project, target) = external_link();
      fs::write(&target, "regular file").expect("create external file target");
      let link = AbsPath::try_from_std_path(&project.join("link")).expect("valid path");
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
        vec![(
          AbsPath::try_from_std_path(&project).expect("valid path"),
          RecursiveMode::NonRecursive,
        )],
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

#[tokio::test]
async fn nested_contexts_route_unscanned_children_after_either_unsubscribes() {
  let temp = TempDir::new().expect("create fixture");
  let outer = AbsPath::try_from_std_path(&temp.path().canonicalize().expect("resolve fixture"))
    .expect("valid utf-8 fixture path");
  let inner = outer.join("inner").expect("valid path");
  fs::create_dir(&inner).expect("create nested context");
  let event = Event::new(EventKind::Remove(notify::event::RemoveKind::File)).add_path(
    inner
      .join("unscanned/child.js")
      .expect("valid path")
      .to_std_path_buf(),
  );

  for (unregistered, remaining) in [(&inner, &outer), (&outer, &inner)] {
    let mut analyzer = DirectoryAnalyzer::default();
    analyzer
      .update(empty(), added(&outer), empty())
      .await
      .expect("register outer context");
    analyzer.on_scanned_change(&inner);
    assert_eq!(
      next_events(&mut analyzer).await,
      vec![(outer.clone(), FsEventKind::Change)]
    );
    analyzer
      .update(empty(), added(&inner), added(&inner))
      .await
      .expect("also register inner as a context and missing dependency");

    analyzer.on_native_event(event.clone());
    assert_eq!(
      next_events(&mut analyzer).await,
      vec![
        (inner.clone(), FsEventKind::Change),
        (outer.clone(), FsEventKind::Change)
      ]
    );
    analyzer
      .update(empty(), removed(unregistered), removed(&inner))
      .await
      .expect("unregister one context and the overlapping missing dependency");
    analyzer.on_native_event(event.clone());
    assert_eq!(
      next_events(&mut analyzer).await,
      vec![(remaining.clone(), FsEventKind::Change)]
    );
  }
}
