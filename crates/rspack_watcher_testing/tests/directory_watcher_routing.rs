use std::{
  fs,
  path::{Path, PathBuf},
  sync::{Arc, Mutex},
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use rspack_paths::InternedPath;
use rspack_util::fx_hash::FxHashSet;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind, FsWatcher, TimeInfoEntry, testing::*,
};

#[derive(Clone, Default)]
struct Rec(Arc<Mutex<Vec<String>>>);
impl EventAggregateHandler for Rec {
  fn on_event_handle(&self, _: FxHashSet<String>, _: FxHashSet<String>) {}
}
impl EventHandler for Rec {
  fn on_change(&self, f: String) -> rspack_error::Result<()> {
    self.0.lock().expect("l").push(f);
    Ok(())
  }
}

fn ip(s: &str) -> InternedPath {
  InternedPath::from(Path::new(s))
}
fn wf(p: &Path, m: SystemTime) {
  fs::write(p, "v").expect("w");
  let f = fs::File::options().write(true).open(p).expect("o");
  f.set_modified(m).expect("m");
}
fn info(w: &FsWatcher, p: &Path) -> (Option<u64>, bool) {
  let (s, e) = (p.to_string_lossy(), w.collect_time_info_entries().0);
  let t = e.iter().find_map(|(n, v)| match v {
    TimeInfoEntry::Entry { timestamp, .. } if n == s.as_ref() => Some(*timestamp),
    _ => None,
  });
  let eo = TimeInfoEntry::ExistenceOnlyTimeEntry;
  (t, e.iter().any(|(n, v)| n == s.as_ref() && *v == eo))
}
async fn u(w: &FsWatcher, r: &Rec, p: [&[&Path]; 6]) {
  let [f0, f1, d0, d1, m0, m1] = p.map(|s| {
    s.iter()
      .map(|x| InternedPath::from(*x))
      .collect::<Vec<_>>()
      .into_iter()
  });
  let (a, b, now) = (Box::new(r.clone()), Box::new(r.clone()), SystemTime::now());
  w.watch((f0, f1), (d0, d1), (m0, m1), now, a, b).await;
}
async fn wait(mut f: impl FnMut() -> bool) {
  let deadline = std::time::Instant::now() + Duration::from_secs(5);
  while !f() {
    assert!(std::time::Instant::now() < deadline, "timeout");
    tokio::time::sleep(Duration::from_millis(15)).await;
  }
}

#[test]
fn routing_normalization_and_directory_watcher_exits() {
  for pair in "/=/ ///=/ /\\=/ \\/=/ ///\\=/ /a/b=/a/b /a/b/=/a/b /a/b///=/a/b C:\\=C:\\ C:\\\\=C:\\ C:/=C:/ C:\\a\\=C:\\a C:/a/=C:/a".split(' ') {
    let (i, e) = pair.split_once('=').expect("p");
    assert_eq!(normalize_dir_path(&ip(i)), ip(e), "{i}");
  }
  let wm = WatcherManager::default();
  let wf_sub = |p: &InternedPath| wm.watch_file(p, WatcherSubKind::File, 1);
  assert_eq!(wf_sub(&ip("/")), None);
  let (src, asset, a_sl) = (ip("/s"), ip("/s/a"), ip("/s/a/"));
  assert_eq!(wf_sub(&asset), Some(src.clone()));
  assert!(!wm.get(&src).expect("s").nested_watching);
  assert_eq!(wm.watch_directory(&a_sl, 2), asset);
  assert!(wm.directories.len() == 2 && wm.get(&asset).expect("dw").nested_watching);
  let ld = || wm.logical_directories(&asset);
  assert_eq!(ld(), vec![a_sl.clone()]);
  wm.watch_directory(&asset, 3);
  assert_eq!(wm.get(&asset).map(|d| d.refs), Some(2));
  assert_eq!(ld(), [a_sl.clone(), asset.clone()]);
  wm.unwatch_directory(&asset);
  assert_eq!(wm.get(&asset).map(|d| d.refs), Some(1));
  wm.unwatch_directory(&a_sl);
  assert!(wm.get(&asset).is_none() && wm.directories.len() == 1);
  wm.unwatch_file(&asset, WatcherSubKind::File);
  assert_eq!(wm.directories.len(), 0);

  let (root, now) = (ip("/r"), SystemTime::now());
  let mut dw = DirectoryWatcher::new(root.clone());
  let aged = now - Duration::from_secs(3600);
  let (miss, ctx, sub, norm, item) = (ip("/r/m"), ip("/r/c"), ip("/r/d/"), ip("/r/d"), ip("/r/i"));
  assert!(dw.set_nested_watching(true) && !dw.set_nested_watching(true));
  assert!(dw.set_file_time(&miss, aged, true, true) && !dw.context_files.contains(&miss));
  assert!(dw.set_file_time(&miss, aged, false, false));
  assert!(dw.set_file_time(&ctx, aged, true, false) && dw.context_files.insert(ctx.clone()));
  let safe = dw_file(&dw, &ctx).1;
  assert!(!dw.set_file_time(&ctx, aged, false, false) && dw_file(&dw, &ctx).1 == safe);
  assert!(dw.set_directory(&sub, aged, true) && !dw.set_directory(&root, aged, true));
  assert!(dw.set_missing(&ctx, false) && dw.removed_times.contains_key(&ctx));
  assert!(!dw_file(&dw, &ctx).0 && dw.set_file_time(&item, aged, true, false));
  assert!(dw.context_files.insert(item.clone()) && dw.set_directory(&item, now, false));
  assert!(!dw_file(&dw, &item).0 && dw.set_missing(&item, false));
  assert!(!dw_file(&dw, &item).0 && !dw.context_files.contains(&item));
  assert!(!dw.directories.contains(&item) && dw.set_file_time(&ctx, now, false, false));
  assert!(!dw.removed_times.contains_key(&ctx) && dw.directories.contains(&norm));
  assert!(dw.directory_safe_times.contains_key(&norm) && dw.on_directory_removed().0 == [norm]);
  assert!(!dw.exists_on_disk && dw.last_watch_event == 0 && dw.directories.is_empty());
}

#[tokio::test]
async fn trailing_slash_and_parent_unsub_preservation() {
  let old = UNIX_EPOCH + Duration::from_secs(1_600_000_000);
  let rep = UNIX_EPOCH + Duration::from_secs(1_600_000_300);
  let sl = |p: &Path| PathBuf::from(format!("{}/", p.display()));
  for mode in ["normal", "slash", "both"] {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().canonicalize().expect("canon");
    let (assets, manifest) = (root.join("assets"), root.join("assets/manifest.json"));
    fs::create_dir(&assets).expect("dir");
    wf(&manifest, old);
    let d = [assets.as_path(), &sl(&assets)];
    let dirs = match mode {
      "normal" => &d[..1],
      "slash" => &d[1..],
      _ => &d[..],
    };
    let w = FsWatcher::new(Default::default(), Default::default());
    let r = Rec::default();
    u(&w, &r, [&[], &[], dirs, &[], &[], &[]]).await;
    assert!(info(&w, &manifest).0.is_some(), "{mode}");
    fs::remove_file(&manifest).expect("rm");
    wait(|| info(&w, &manifest).0.is_none()).await;
    assert!(watcher_dw_files_empty(&w, &InternedPath::from(&assets)));
    if mode == "both" {
      let (stage, icons) = (root.join("stage"), assets.join("icons"));
      let isl = sl(&icons);
      wf(&stage, old);
      u(&w, &r, [&[], &[], &[&isl], &[], &[], &[]]).await;
      fs::rename(&stage, &icons).expect("rename");
      wait(|| info(&w, &icons).0.is_some()).await;
      let ic: &[&Path] = &[&icons];
      u(&w, &r, [ic, &[], &[], &[&assets, &isl], &[], &[]]).await;
      u(&w, &r, [&[], ic, &[], &[d[1]], &[], &[]]).await;
      assert_eq!(watcher_counts(&w).0, 0);
      wf(&icons, rep);
      u(&w, &r, [ic, &[], &[], &[], &[], &[]]).await;
      assert_eq!(info(&w, &icons).0, Some(1_600_000_300_000));
    } else if mode == "normal" {
      let j = |s| root.join(s);
      let (d1, d2, out) = (j("d1"), j("d1/d2"), j("out"));
      let (man, miss, sen) = (j("out/m"), j("out/x"), j("s"));
      let _ = [&d2, &out].map(|d| fs::create_dir_all(d).expect("d"));
      let _ = [&man, &miss, &sen].map(|p| wf(p, old));
      u(&w, &r, [&[], &[], &[&d1, &d2], &[&assets], &[], &[]]).await;
      assert_eq!(watcher_counts(&w).0, 2);
      u(&w, &r, [&[], &[], &[], &[&d1], &[], &[]]).await;
      assert_eq!(watcher_counts(&w).0, 1);
      u(&w, &r, [&[&man, &sen], &[], &[&out], &[&d2], &[&miss], &[]]).await;
      assert!(info(&w, &miss).0.is_some());
      u(&w, &r, [&[], &[], &[], &[&out], &[], &[]]).await;
      assert!(info(&w, &miss).0.is_some());
      wf(&sen, old + Duration::from_secs(60));
      let ev = |p: &Path| w.trigger_event(&InternedPath::from(p), FsEventKind::Change);
      ev(&man);
      ev(&sen);
      let has = |p: &Path| r.0.lock().expect("l").iter().any(|x| Path::new(x) == p);
      wait(|| has(&sen)).await;
      assert!(!has(&man));
      u(&w, &r, [&[], &[&man, &sen], &[], &[], &[], &[&miss]]).await;
      assert_eq!(watcher_counts(&w).0, 0);
      let (item, sub) = (out.join("item"), out.join("sub"));
      let _ = [&item, &sub].map(|p| wf(p, old));
      u(&w, &r, [&[], &[], &[&out], &[], &[&item], &[]]).await;
      for p in [&item, &sub] {
        fs::remove_file(p)
          .and_then(|_| fs::create_dir(p))
          .expect("d");
        ev(p);
      }
      wait(|| info(&w, &item).1 && info(&w, &sub).1).await;
      assert!(info(&w, &item).0.is_none());
      u(&w, &r, [&[], &[], &[&sub], &[&out], &[], &[]]).await;
      assert!(info(&w, &item).0.is_none());
      u(&w, &r, [&[], &[], &[], &[], &[], &[&item]]).await;
      assert_eq!(watcher_counts(&w).0, 1);
      u(&w, &r, [&[], &[], &[], &[&sub], &[], &[]]).await;
      assert_eq!(watcher_counts(&w), (0, (0, 0)));
      #[cfg(unix)]
      {
        let (outer, target, link) = (j("outer"), j("target"), j("outer/link"));
        let _ = [&outer, &target].map(|d| fs::create_dir_all(d).expect("d"));
        wf(&target.join("tf.js"), old);
        std::os::unix::fs::symlink(&target, &link).expect("sym");
        u(&w, &r, [&[], &[], &[&outer, &sl(&link)], &[], &[], &[]]).await;
        u(&w, &r, [&[], &[], &[], &[&outer], &[], &[]]).await;
        assert!(info(&w, &link).0.is_some() && watcher_counts(&w).0 == 2);
        u(&w, &r, [&[], &[], &[], &[&sl(&link)], &[], &[]]).await;
        assert_eq!(watcher_counts(&w), (0, (0, 0)));
      }
    }
    w.close().await.expect("close");
  }
}
