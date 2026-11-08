#![allow(dead_code)]

//! The filesystem accuracy is process-wide and only narrows, so this test runs
//! in its own binary: nothing has narrowed it before it starts.

use rspack_watcher::{FsWatcher, FsWatcherOptions, TimeInfoEntry};

mod helpers;

fn set_modified(path: &std::path::Path, when: std::time::SystemTime) -> std::io::Result<()> {
  let mut options = std::fs::File::options();
  #[cfg(windows)]
  {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    options.write(true).custom_flags(FILE_FLAG_BACKUP_SEMANTICS);
  }
  #[cfg(not(windows))]
  options.read(true);
  options.open(path)?.set_modified(when)
}

fn accuracy_of(helper: &helpers::TestHelper, name: &str) -> (u64, u64, u64) {
  let (file_timestamps, _) = helper.collect_time_info_entries();
  match *helper.time_info_entry(&file_timestamps, name) {
    TimeInfoEntry::Entry {
      safe_time,
      timestamp,
      accuracy,
    } => (safe_time, timestamp, accuracy),
    ref other => panic!("{name} is not a file entry: {other:?}"),
  }
}

/// Like watchpack's `fixupEntryAccuracy`, a record taken while the accuracy
/// was still coarse is read with the narrower accuracy learned since.
#[test]
fn a_recorded_accuracy_narrows_with_the_filesystem_accuracy() {
  let mut helper = helpers::TestHelper::new(|| {
    FsWatcher::new(
      FsWatcherOptions {
        aggregate_timeout: Some(100),
        ..Default::default()
      },
      Default::default(),
    )
  });
  // Whole seconds on an even boundary teach nothing: the accuracy stays 2s.
  let round = {
    let millis = (std::time::SystemTime::now() - std::time::Duration::from_secs(3600))
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis() as u64;
    std::time::UNIX_EPOCH + std::time::Duration::from_millis(millis / 2000 * 2000)
  };
  std::fs::create_dir_all(helper.join("ctx")).unwrap();
  helper.file("ctx/coarse");
  for name in ["ctx/coarse", "ctx"] {
    set_modified(helper.join(name).as_std_path(), round).unwrap();
  }

  let _rx = helper.watch(
    (std::iter::empty(), std::iter::empty()),
    (
      vec![rspack_paths::InternedPath::from("ctx")].into_iter(),
      std::iter::empty(),
    ),
    (std::iter::empty(), std::iter::empty()),
  );
  std::thread::sleep(std::time::Duration::from_millis(300));
  let (_, _, accuracy) = accuracy_of(&helper, "ctx/coarse");
  assert_eq!(
    accuracy, 2000,
    "precondition: nothing narrowed the accuracy yet"
  );

  // A new context's scan of a file on an odd millisecond narrows it to 1ms.
  std::fs::create_dir_all(helper.join("ctx2")).unwrap();
  helper.file("ctx2/fine");
  set_modified(
    helper.join("ctx2/fine").as_std_path(),
    round + std::time::Duration::from_millis(1),
  )
  .unwrap();
  let _rx = helper.watch(
    (std::iter::empty(), std::iter::empty()),
    (
      vec![rspack_paths::InternedPath::from("ctx2")].into_iter(),
      std::iter::empty(),
    ),
    (std::iter::empty(), std::iter::empty()),
  );

  let (safe_time, timestamp, accuracy) = accuracy_of(&helper, "ctx/coarse");
  assert_eq!(accuracy, 1, "the coarse record keeps a 2s accuracy");
  assert_eq!(safe_time, timestamp + 1);
}
