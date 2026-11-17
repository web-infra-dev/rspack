use std::{fs, time::Duration};

use notify::{Event, EventKind, event::CreateKind};
use rspack_paths::InternedPath;
use rspack_watcher::{FsEventKind, testing::DirectoryAnalyzer};
use tempfile::TempDir;

async fn next_events(analyzer: &mut DirectoryAnalyzer) -> Vec<(InternedPath, FsEventKind)> {
  tokio::time::timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .expect("report directory change")
}

async fn check_repeated_directory_event(register_child: bool) {
  for kind in [
    EventKind::Create(CreateKind::Folder),
    EventKind::Create(CreateKind::Any),
  ] {
    let fixture = TempDir::new().expect("create fixture");
    let parent = InternedPath::from(fixture.path().canonicalize().expect("resolve fixture"));
    let child = InternedPath::from(parent.join("child"));
    fs::create_dir(&child).expect("create child");
    let mut analyzer = DirectoryAnalyzer::default();
    analyzer
      .update(
        (vec![], vec![]),
        (vec![parent.clone()], vec![]),
        (vec![], vec![]),
      )
      .await
      .expect("register parent");
    analyzer.on_native_event(Event::new(kind).add_path(child.as_ref().to_path_buf()));
    assert_eq!(
      next_events(&mut analyzer).await,
      vec![(parent.clone(), FsEventKind::Change)]
    );

    if register_child {
      analyzer
        .update(
          (vec![], vec![]),
          (vec![child.clone()], vec![]),
          (vec![], vec![]),
        )
        .await
        .expect("register child");
    }
    analyzer.on_native_event(Event::new(kind).add_path(child.as_ref().to_path_buf()));
    if register_child {
      assert_eq!(
        next_events(&mut analyzer).await,
        vec![(child.clone(), FsEventKind::Create)],
        "{kind:?}"
      );
    } else {
      assert!(
        tokio::time::timeout(Duration::from_millis(200), analyzer.next_events())
          .await
          .is_err(),
        "a repeated {kind:?} must not report an existing entry as new"
      );
    }

    fs::rename(&child, parent.join("previous")).expect("move old child");
    fs::create_dir(&child).expect("replace child");
    analyzer.on_native_event(Event::new(kind).add_path(child.as_ref().to_path_buf()));
    assert!(
      next_events(&mut analyzer)
        .await
        .contains(&(parent, FsEventKind::Change)),
      "replacement must notify the parent for {kind:?}"
    );
  }
}

#[tokio::test]
async fn repeated_directory_creation_does_not_notify_parent() {
  check_repeated_directory_event(false).await;
}

#[tokio::test]
async fn repeated_directory_creation_keeps_its_own_subscription() {
  check_repeated_directory_event(true).await;
}
