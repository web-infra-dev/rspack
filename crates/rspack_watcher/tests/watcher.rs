#![allow(dead_code)]

use std::sync::atomic::AtomicU8;

use rspack_paths::ArcPath;
use rspack_watcher::{FsWatcher, FsWatcherOptions};

mod helpers;

macro_rules! e {
  () => {
    (std::iter::empty(), std::iter::empty())
  };
}

macro_rules! f {
  ($($file:expr),*) => {
    (vec![$(ArcPath::from($file)),*].into_iter(), std::iter::empty())
  };
}

macro_rules! h {
  ($options:expr) => {
    h!($options, Default::default())
  };
  ($options:expr, $ignore:expr) => {
    helpers::TestHelper::new(|| FsWatcher::new($options, $ignore))
  };
}

macro_rules! c {
  () => {
    AtomicU8::new(0)
  };
}

macro_rules! add {
  ($c:expr) => {
    $c.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
  };
}

macro_rules! load {
  ($c:expr) => {
    $c.load(std::sync::atomic::Ordering::SeqCst)
  };
}

macro_rules! watch {
  ($helper:expr, $($files:expr),*) => {
    watch!(files @ $helper, $($files),*)
  };
  ($helper:expr, _, $($dirs:expr),*) => {
    watch!(dirs @ $helper, $($dirs),*)
  };
  ($helper:expr, _, _, $($missing:expr),*) => {
    watch!(missing @ $helper, $($missing),*)
  };

  (files @ $helper:expr, $($files:expr),*) => {
    $helper.watch(f!($($files),*), e!(), e!())
  };
  (dirs @ $helper:expr, $($dirs:expr),*) => {
    $helper.watch(e!(), f!($($dirs),*), e!())
  };
  (missing @ $helper:expr, $($missing:expr),*) => {
    $helper.watch(e!(), e!(), f!($($missing),*))
  };
}

#[test]
fn should_watch_a_single_file() {
  let mut helper = h!(FsWatcherOptions {
    aggregate_timeout: Some(1000),
    ..Default::default()
  });

  let rx = watch!(helper, "a");

  helper.tick(|| {
    helper.file("a");
  });

  let change_events = c!();
  helper.collect_events(
    rx,
    |file, _| {
      file.assert_path(helper.join("a"));
      add!(change_events);
    },
    |changes, abort| {
      changes.assert_changed(helper.join("a"));
      assert!(load!(change_events) > 0);
      *abort = true;
    },
  );
}
