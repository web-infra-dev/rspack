// webpack: "timestamp mode" in FileSystemInfo.unittest.js (L318-L449).
use super::*;

const OPTIONS: SnapshotStrategyOptions = SnapshotStrategyOptions::timestamp();

#[tokio::test]
async fn should_always_accept_an_empty_snapshot() {
  check_empty_snapshot(OPTIONS).await;
}

#[tokio::test]
async fn should_accept_a_snapshot_when_fs_is_unchanged() {
  check_unchanged_snapshot(OPTIONS).await;
}

mod invalidates_when_changed {
  use super::*;

  #[tokio::test]
  async fn file() {
    check_change(OPTIONS, "/path/file.txt", false, false).await;
  }

  #[tokio::test]
  async fn file2() {
    check_change(OPTIONS, "/path/file2.txt", false, false).await;
  }

  #[tokio::test]
  async fn nested_deep_file() {
    check_change(OPTIONS, "/path/nested/deep/file.txt", false, false).await;
  }

  #[tokio::test]
  async fn context_files_file() {
    check_change(OPTIONS, "/path/context+files/file.txt", false, false).await;
  }

  #[tokio::test]
  async fn context_files_file2() {
    check_change(OPTIONS, "/path/context+files/file2.txt", false, false).await;
  }

  #[tokio::test]
  async fn context_files_sub_file() {
    check_change(OPTIONS, "/path/context+files/sub/file.txt", false, false).await;
  }

  #[tokio::test]
  async fn context_files_sub_file2() {
    check_change(OPTIONS, "/path/context+files/sub/file2.txt", false, false).await;
  }

  #[tokio::test]
  async fn context_files_sub_file3() {
    check_change(OPTIONS, "/path/context+files/sub/file3.txt", false, false).await;
  }

  #[tokio::test]
  async fn context_file() {
    check_change(OPTIONS, "/path/context/file.txt", false, false).await;
  }

  #[tokio::test]
  async fn context_file2() {
    check_change(OPTIONS, "/path/context/file2.txt", false, false).await;
  }

  #[tokio::test]
  async fn context_sub_file() {
    check_change(OPTIONS, "/path/context/sub/file.txt", false, false).await;
  }

  #[tokio::test]
  async fn context_sub_file2() {
    check_change(OPTIONS, "/path/context/sub/file2.txt", false, false).await;
  }

  #[tokio::test]
  async fn context_sub_file3() {
    check_change(OPTIONS, "/path/context/sub/file3.txt", false, false).await;
  }

  #[tokio::test]
  async fn managed_package_json() {
    check_change(
      OPTIONS,
      "/path/node_modules/package/package.json",
      false,
      false,
    )
    .await;
  }

  #[tokio::test]
  async fn symlink_context_target() {
    check_change(OPTIONS, "/path/folder/context/file.txt", false, false).await;
  }

  #[tokio::test]
  async fn symlink_context_files_target() {
    check_change(OPTIONS, "/path/folder/context+files/file.txt", false, false).await;
  }

  #[tokio::test]
  async fn symlink_nested_target() {
    check_change(OPTIONS, "/path/folder/nested/file.txt", false, false).await;
  }

  #[tokio::test]
  async fn unmanaged_scoped_package1() {
    check_change(
      OPTIONS,
      "/path/node_modules/@foo/package1/index.js",
      false,
      false,
    )
    .await;
  }

  #[tokio::test]
  async fn unmanaged_scoped_package2() {
    check_change(
      OPTIONS,
      "/path/node_modules/@foo/package2/index.js",
      false,
      false,
    )
    .await;
  }

  #[tokio::test]
  async fn unmanaged_package3() {
    check_change(
      OPTIONS,
      "/path/node_modules/bar-package3/index.js",
      false,
      false,
    )
    .await;
  }
}

mod stays_valid_when_changed {
  use super::*;

  #[tokio::test]
  async fn managed_file() {
    check_change(OPTIONS, "/path/node_modules/package/file.txt", false, true).await;
  }

  #[tokio::test]
  async fn managed_ignored_file() {
    check_change(
      OPTIONS,
      "/path/node_modules/package/ignored.txt",
      false,
      true,
    )
    .await;
  }

  #[tokio::test]
  async fn immutable_package_json() {
    check_change(
      OPTIONS,
      "/path/cache/package-1234/package.json",
      false,
      true,
    )
    .await;
  }

  #[tokio::test]
  async fn immutable_file() {
    check_change(OPTIONS, "/path/cache/package-1234/file.txt", false, true).await;
  }

  #[tokio::test]
  async fn immutable_ignored_file() {
    check_change(OPTIONS, "/path/cache/package-1234/ignored.txt", false, true).await;
  }
}

mod invalidates_when_created {
  use super::*;

  #[tokio::test]
  async fn package_json() {
    check_change(OPTIONS, "/path/package.json", true, false).await;
  }

  #[tokio::test]
  async fn file2() {
    check_change(OPTIONS, "/path/file2.txt", true, false).await;
  }

  #[tokio::test]
  async fn context_files_file2() {
    check_change(OPTIONS, "/path/context+files/file2.txt", true, false).await;
  }

  #[tokio::test]
  async fn managed_package_file() {
    check_change(OPTIONS, "/path/node_modules/package.txt", true, false).await;
  }
}

mod stays_valid_when_created {
  use super::*;

  #[tokio::test]
  async fn managed_missing_file() {
    check_change(
      OPTIONS,
      "/path/node_modules/package/missing.txt",
      true,
      true,
    )
    .await;
  }

  #[tokio::test]
  async fn immutable_missing_file() {
    check_change(OPTIONS, "/path/cache/package-1234/missing.txt", true, true).await;
  }

  #[tokio::test]
  async fn immutable_package() {
    check_change(OPTIONS, "/path/cache/package-2345", true, true).await;
  }

  #[test]
  #[should_panic(expected = "not yet implemented: webpack L428:")]
  fn ignored_missing_file() {
    todo!(
      "webpack L428: create /path/ignored.txt after snapshotting; requires addFileTimestamps and the ignore state for missing dependencies"
    );
  }
}

#[test]
#[should_panic(expected = "not yet implemented: webpack L397:")]
fn should_not_invalidate_when_nested_deep_ignored_file_changes() {
  todo!(
    "webpack L397: /path/nested/deep/ignored.txt requires addFileTimestamps(ignore); do not replace watcher ignore with immutablePaths"
  );
}
#[test]
#[should_panic(expected = "not yet implemented: webpack L397:")]
fn should_not_invalidate_when_context_files_sub_ignored_file_changes() {
  todo!(
    "webpack L397: /path/context+files/sub/ignored.txt requires addFileTimestamps(ignore) in file and context timestamp reads"
  );
}
#[test]
#[should_panic(expected = "not yet implemented: webpack L397:")]
fn should_not_invalidate_when_context_sub_ignored_file_changes() {
  todo!(
    "webpack L397: /path/context/sub/ignored.txt requires context timestamp aggregation to skip ignored file timestamps"
  );
}
