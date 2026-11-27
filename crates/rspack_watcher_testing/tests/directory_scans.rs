use std::{fs, time::Duration};

use rspack_watcher::testing::DirectoryAnalyzer;
use tokio::time::timeout;

#[tokio::test]
async fn initial_and_live_directories_batch_metadata_on_the_registration_pool() {
  use notify::{Event, EventKind, event::CreateKind};
  use tokio::runtime::Handle;

  let temp = tempfile::tempdir().unwrap();
  let root = temp.path().canonicalize().unwrap();
  let watched = root.join("watched");
  let staging = root.join("staging");
  for dir in [&watched, &staging] {
    fs::create_dir(dir).unwrap();
    for i in 0..256 {
      fs::write(dir.join(format!("{i}.js")), "").unwrap();
    }
  }
  let mut analyzer = DirectoryAnalyzer::default();
  analyzer
    .update(
      Default::default(),
      (vec![(&watched).into()], vec![]),
      Default::default(),
    )
    .await
    .unwrap();
  let owner = Handle::current().id();
  let check = |analyzer: &DirectoryAnalyzer| {
    let jobs = analyzer.take_metadata_jobs();
    assert_eq!(jobs.iter().map(|(files, _)| files).sum::<usize>(), 256);
    assert!(jobs.len() < 256, "one blocking job for each file");
    assert!(jobs.iter().all(|(_, runtime)| *runtime == owner));
  };
  analyzer.scan_contexts().await.unwrap();
  check(&analyzer);

  let moved = watched.join("moved");
  fs::rename(&staging, &moved).unwrap();
  analyzer.on_native_event(Event::new(EventKind::Create(CreateKind::Folder)).add_path(moved));
  timeout(Duration::from_secs(5), analyzer.next_events())
    .await
    .unwrap();
  check(&analyzer);
}
