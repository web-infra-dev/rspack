use notify::RecursiveMode::{NonRecursive, Recursive};
use rspack_paths::InternedPath;
use rspack_watcher::testing::WatchInstallations;

#[test]
fn failed_recursive_upgrade_keeps_the_old_watch_and_can_be_retried() {
  let root = InternedPath::from(std::path::Path::new("root"));
  let shallow = [(root.clone(), NonRecursive)];
  let both = [(root.clone(), Recursive), (root.clone(), NonRecursive)];
  let mut watches = WatchInstallations::default();
  let mut installed = None;
  watches
    .update(&shallow, |_, mode| {
      installed = mode;
      Ok(())
    })
    .expect("install shallow watch");

  let mut calls = Vec::new();
  let failure = watches.update(&both, |path, mode| {
    assert_eq!(path, &root);
    calls.push(mode);
    if mode == Some(Recursive) {
      return Err(notify::Error::generic("failed to install"));
    }
    installed = mode;
    Ok(())
  });
  assert!(failure.is_err());
  assert_eq!(installed, Some(NonRecursive));
  assert_eq!(calls, [None, Some(Recursive), Some(NonRecursive)]);

  calls.clear();
  assert!(
    watches
      .update(&both, |_, mode| {
        calls.push(mode);
        installed = mode;
        Ok(())
      })
      .expect("retry upgrade")
  );
  assert_eq!(calls, [None, Some(Recursive)]);
  assert_eq!(installed, Some(Recursive));
  assert!(
    !watches
      .update(&both, |_, _| panic!("already installed"))
      .expect("reuse watch")
  );
  assert!(
    watches
      .update(&shallow, |_, mode| {
        installed = mode;
        Ok(())
      })
      .expect("downgrade")
  );
  assert_eq!(installed, Some(NonRecursive));
}
